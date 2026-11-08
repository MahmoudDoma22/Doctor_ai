use crate::{catalog::{self, MODELS}, download::{self, DownloadEvent}};
use eframe::egui;
use serde_json::json;
use std::{io::{Read,Write},net::{TcpListener,TcpStream},process::{Child,Command,Stdio},sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use tokio::sync::mpsc;

#[derive(PartialEq,Clone,Copy)] enum Page{Welcome,Chat,Models,Downloads,Tools,Settings}
pub struct DoctorApp{
 page:Page,ram:u64,available_ram:u64,free_disk:u64,cpu:String,selected:usize,notices:Vec<String>,
 download_rx:mpsc::UnboundedReceiver<DownloadEvent>,download_tx:mpsc::UnboundedSender<DownloadEvent>,
 cancel:Arc<AtomicBool>,downloading:bool,download_progress:f32,download_label:String,
 prompt:String,answer:String,busy:bool,stop:Arc<AtomicBool>,engine:Option<Child>,
 status:String,chat_rx:Option<std::sync::mpsc::Receiver<String>>,download_speed:String,last_progress:Option<(u64,std::time::Instant)>,
 http:reqwest::Client,engine_port:u16,engine_ready:bool,engine_started_at:Option<Instant>,
 installed_models:Vec<std::path::PathBuf>,installed_tools:Vec<(&'static str,bool)>, 
}
impl DoctorApp{
 pub fn new()->Self{
  let(tx,rx)=mpsc::unbounded_channel();let installed_models=catalog::installed_model_files();let installed_tools=catalog::installed_tools();let ram=catalog::detect_ram_gb();let available_ram=catalog::detect_available_ram_gb();let free_disk=catalog::detect_free_disk_gb();let cpu=catalog::detect_cpu();
  Self{page:Page::Welcome,ram,available_ram,free_disk,cpu,selected:catalog::recommended(ram,available_ram),notices:vec!["مرحباً بك في Doctor AI".into()],download_rx:rx,download_tx:tx,cancel:Arc::new(AtomicBool::new(false)),downloading:false,download_progress:0.0,download_label:String::new(),prompt:String::new(),answer:String::new(),busy:false,stop:Arc::new(AtomicBool::new(false)),engine:None,status:"جاهز".into(),chat_rx:None,download_speed:"".into(),last_progress:None,http:reqwest::Client::builder().pool_max_idle_per_host(4).connect_timeout(Duration::from_secs(5)).build().unwrap_or_default(),engine_port:0,engine_ready:false,engine_started_at:None,installed_models,installed_tools}
 }
 fn model_present(&self,m:&catalog::Model)->bool{
  self.installed_models.iter().any(|p|p.file_name().and_then(|n|n.to_str()).map(|n|n.eq_ignore_ascii_case(m.file_name)).unwrap_or(false))
 }
 fn start_model_download(&mut self){
  let m=&MODELS[self.selected];self.cancel.store(false,Ordering::Relaxed);self.downloading=true;self.download_progress=0.0;
  self.download_label=format!("تنزيل {}",m.name);self.status="جاري تنزيل الموديل — الاستكمال مدعوم بعد إغلاق التطبيق".into();
  download::start_download(m.url.into(),catalog::model_path(m),self.cancel.clone(),self.download_tx.clone());
 }
 fn start_engine(&mut self){
  let Some(path)=catalog::find_model_path(&MODELS[self.selected]) else {self.status="الموديل غير موجود؛ افتح صفحة الموديلات لتنزيله".into();return;};
  let exe=std::env::current_exe().ok().and_then(|p|p.parent().map(|p|p.join("runtime").join("mistralrs").join("mistralrs.exe")));
  let Some(exe)=exe.filter(|p|p.exists())else{self.status="محرك Rust غير موجود في runtime/mistralrs".into();return;};
  let log_path=catalog::model_dir().parent().unwrap_or_else(||std::path::Path::new(".")).join("mistralrs.log");
  let listener=match TcpListener::bind(("127.0.0.1",0)){Ok(l)=>l,Err(e)=>{self.status=format!("تعذر حجز منفذ محلي للمحرك: {e}");return;}};
  self.engine_port=listener.local_addr().map(|a|a.port()).unwrap_or(0);drop(listener);
  if self.engine_port==0{self.status="تعذر تحديد منفذ محلي للمحرك".into();return;}
  if let Some(parent)=log_path.parent(){if let Err(e)=std::fs::create_dir_all(parent){self.status=format!("تعذر إنشاء مجلد السجل {}: {e}",parent.display());return;}}
  let log=match std::fs::OpenOptions::new().create(true).append(true).open(&log_path){Ok(f)=>f,Err(e)=>{self.status=format!("تعذر فتح سجل المحرك {}: {e}",log_path.display());return;}};
  let stdout=match log.try_clone(){Ok(f)=>Stdio::from(f),Err(e)=>{self.status=format!("تعذر تجهيز سجل المحرك {}: {e}",log_path.display());return;}};
  match Command::new(exe).args(["serve","-f"]).arg(path).args(["--host","127.0.0.1","--port",&self.engine_port.to_string(),"--no-ui"]).stdout(stdout).stderr(Stdio::from(log)).spawn(){Ok(c)=>{self.engine=Some(c);self.engine_ready=false;self.engine_started_at=Some(Instant::now());self.status=format!("جاري تحميل الموديل والتحقق من جاهزية المحرك على المنفذ {}؛ السجل: {}",self.engine_port,log_path.display());},Err(e)=>self.status=format!("فشل تشغيل المحرك؛ السجل: {} — {e}",log_path.display())}
 }
 fn send_prompt(&mut self){
  if self.prompt.trim().is_empty()||self.busy{return;}let prompt=std::mem::take(&mut self.prompt);
  if !self.engine_ready{self.status="المحرك لم يصبح جاهزاً بعد؛ انتظر اكتمال تحميل الموديل".into();return;}
  self.answer.clear();self.busy=true;self.stop.store(false,Ordering::Relaxed);self.status="جاري الاتصال بالموديل المحلي...".into();
  let stop=self.stop.clone();let http=self.http.clone();let port=self.engine_port;let(tx,rx)=std::sync::mpsc::channel();let tx_error=tx.clone();self.chat_rx=Some(rx);
  std::thread::spawn(move||{
   let rt=tokio::runtime::Builder::new_multi_thread().enable_all().build();
   let result=match rt{Ok(rt)=>rt.block_on(async move{
    let body=json!({"model":"default","stream":true,"messages":[{"role":"user","content":prompt}]});
    let mut response=http.post(format!("http://127.0.0.1:{port}/v1/chat/completions")).json(&body).send().await.map_err(|e|format!("الاتصال بالمحرك المحلي: {e}"))?;
    if !response.status().is_success(){return Err(format!("المحرك رجع HTTP {}",response.status()));}
    let mut buffer=String::new();
    loop {
     let next = tokio::select! {
      _ = tokio::time::sleep(Duration::from_millis(100)) => {
       if stop.load(Ordering::Relaxed) { break; }
       continue;
      }
      chunk = response.chunk() => chunk.map_err(|e|format!("استقبال البيانات: {e}"))?,
     };
     let Some(chunk)=next else { break; };
     buffer.push_str(&String::from_utf8_lossy(&chunk));
     while let Some(pos)=buffer.find('\n'){let line=buffer[..pos].trim().to_string();buffer.drain(..=pos);
      if let Some(data)=line.strip_prefix("data:").map(str::trim){if data=="[DONE]"{return Ok::<(),String>(());}if let Ok(v)=serde_json::from_str::<serde_json::Value>(data){if let Some(s)=v["choices"][0]["delta"]["content"].as_str(){let _=tx.send(s.to_string());}}}
     }
    }Ok::<(),String>(())
   }),Err(e)=>Err(format!("تهيئة Tokio: {e}"))};
   if let Err(e)=result{let _=tx_error.send(format!("\n[خطأ] {e}"));}
  });
 }
 fn poll(&mut self){
  while let Ok(ev)=self.download_rx.try_recv(){match ev{
   DownloadEvent::Progress(n,total)=>{let now=std::time::Instant::now();if let Some((old,at))=self.last_progress{let secs=now.duration_since(at).as_secs_f64().max(0.001);self.download_speed=format!("{}/s",bytes(((n.saturating_sub(old))as f64/secs)as u64));}self.last_progress=Some((n,now));self.download_progress=total.map(|t|(n as f32/t.max(1)as f32).clamp(0.0,1.0)).unwrap_or(0.0);self.download_label=format!("{} / {}",bytes(n),total.map(bytes).unwrap_or_else(||"الحجم غير معروف".into()));}
   DownloadEvent::Finished(p)=>{self.downloading=false;if !self.installed_models.contains(&p){self.installed_models.push(p.clone());}self.status="اكتمل التنزيل".into();self.notices.push(format!("اكتمل تنزيل {}",p.display()));}
   DownloadEvent::Error(e)=>{self.downloading=false;self.status=format!("خطأ: {e}");self.notices.push(self.status.clone());}
   DownloadEvent::Cancelled=>{self.downloading=false;self.status="تم إيقاف التنزيل؛ الملف الجزئي محفوظ".into();}
  }}
  let child_exit=self.engine.as_mut().map(|child|child.try_wait());
  match child_exit{Some(Ok(Some(code)))=>{self.engine_ready=false;self.engine=None;self.status=format!("محرك Rust توقف برمز {code}. راجع سجل mistralrs.log في مجلد .doctor-ai");},Some(Err(e))=>{self.engine_ready=false;self.status=format!("تعذر فحص عملية المحرك: {e}");},Some(Ok(None))=>{let ready=probe_engine(self.engine_port);if ready&&!self.engine_ready{self.status="المحرك المحلي جاهز — يمكنك بدء المحادثة".into();}self.engine_ready=ready;if !ready{if let Some(start)=self.engine_started_at{if start.elapsed()>Duration::from_secs(180){self.status="تأخر تحميل الموديل أكثر من 3 دقائق؛ راجع سجل mistralrs.log وتأكد من الذاكرة".into();}}}},None=>{self.engine_ready=false;}}
  let mut disconnected=false;if let Some(rx)=&self.chat_rx{loop{match rx.try_recv(){Ok(s)=>self.answer.push_str(&s),Err(std::sync::mpsc::TryRecvError::Empty)=>break,Err(std::sync::mpsc::TryRecvError::Disconnected)=>{disconnected=true;break;}}}}
  if disconnected{self.chat_rx=None;self.busy=false;if !self.stop.load(Ordering::Relaxed){self.status="اكتمل الرد".into();}}
  if self.stop.load(Ordering::Relaxed)&&self.busy{self.busy=false;self.chat_rx=None;self.status="تم إيقاف الرد".into();}
 }
 fn nav(&mut self,ui:&mut egui::Ui,p:Page,label:&str){if ui.selectable_label(self.page==p,label).clicked(){self.page=p;}}
}
impl eframe::App for DoctorApp{
 fn update(&mut self,ctx:&egui::Context,_:&mut eframe::Frame){
  self.poll();ctx.request_repaint_after(Duration::from_millis(80));
  egui::TopBottomPanel::top("header").show(ctx,|ui|{ui.horizontal(|ui|{ui.heading("✚ Doctor AI");ui.separator();ui.label("ذكاء محلي • خصوصيتك أولاً");ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(format!("{} GB RAM",self.ram));});});});
  egui::SidePanel::left("nav").resizable(false).default_width(190.0).show(ctx,|ui|{ui.add_space(12.0);ui.heading("القائمة");ui.separator();self.nav(ui,Page::Welcome,"⌂  البداية");self.nav(ui,Page::Chat,"✧  المحادثة");self.nav(ui,Page::Models,"▣  الموديلات");self.nav(ui,Page::Downloads,"⇩  التنزيلات");self.nav(ui,Page::Tools,"▤  الصور والفيديو");self.nav(ui,Page::Settings,"⚙  الإعدادات");ui.with_layout(egui::Layout::bottom_up(egui::Align::Min),|ui|{ui.separator();ui.label("الإشعارات");for n in self.notices.iter().rev().take(4){ui.label(egui::RichText::new(n).small().weak());}});});
  egui::TopBottomPanel::bottom("status").show(ctx,|ui|{ui.horizontal(|ui|{ui.label("●");ui.label(&self.status);ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(format!("CPU: {}",self.cpu));});});});
  egui::CentralPanel::default().show(ctx,|ui|match self.page{
   Page::Welcome=>{ui.add_space(24.0);egui::Frame::group(ui.style()).fill(egui::Color32::from_rgb(14,34,52)).rounding(egui::Rounding::same(18.0)).inner_margin(egui::Margin::same(24.0)).show(ui,|ui|{ui.horizontal(|ui|{ui.heading(egui::RichText::new("✚").size(48.0).color(egui::Color32::from_rgb(56,211,190)));ui.vertical(|ui|{ui.heading(egui::RichText::new("DOCTOR AI").size(30.0).strong());ui.label("مساعدك الذكي المحلي — بدون Ollama وبدون اعتماد سحابي للموديلات المحلية");});});ui.add_space(10.0);ui.label("واجهة Rust أصلية • تشغيل محلي • تحكم في الموديلات • خصوصية بياناتك");});ui.add_space(16.0);ui.heading("مرحباً بك — لنجهز جهازك");ui.label("فحص تلقائي للمواصفات، اكتشاف الموديلات المثبتة، ثم اختيار الأنسب قبل أول محادثة.");ui.add_space(10.0);egui::Frame::group(ui.style()).show(ui,|ui|{ui.heading("فحص الجهاز");ui.columns(2,|cols|{cols[0].label(format!("الذاكرة الكلية: {} GB",self.ram));cols[0].label(format!("الذاكرة المتاحة: {} GB",self.available_ram));cols[1].label(format!("المساحة المتاحة: {} GB",self.free_disk));cols[1].label(format!("المعالج: {}",self.cpu));});ui.separator();let m=&MODELS[catalog::recommended(self.ram,self.available_ram)];ui.strong(format!("الاختيار المقترح: {}",m.name));ui.label(m.description);ui.label(if self.model_present(m){"الموديل موجود في المسارات المحلية المعروفة ✓"}else{"الموديل غير موجود — يمكنك تنزيله مع دعم الاستكمال"});});ui.add_space(12.0);ui.horizontal(|ui|{if ui.button("١ — اختيار الموديل وتنزيله").clicked(){self.page=Page::Models;}if ui.button("٢ — فتح المحادثة").clicked(){self.page=Page::Chat;}});ui.add_space(8.0);ui.label(format!("تم اكتشاف {} ملف موديل GGUF و{} أداة محلية.",self.installed_models.len(),self.installed_tools.iter().filter(|(_,v)|*v).count()));},
   Page::Models=>{ui.heading("مكتبة الموديلات");ui.label(format!("الجهاز: {} GB RAM • {}",self.ram,self.cpu));for(i,m)in MODELS.iter().enumerate(){egui::Frame::group(ui.style()).show(ui,|ui|{ui.horizontal(|ui|{ui.radio_value(&mut self.selected,i,"");ui.vertical(|ui|{ui.strong(m.name);ui.label(m.description);ui.label(format!("ذاكرة موصى بها: {} GB • الحجم: {} • {}",m.ram_gb,m.size,m.tier));});ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if catalog::present(m){ui.label("موجود ✓");}else if ui.button("فتح صفحة المصدر").clicked(){let _=open::that(m.url);}});});});}ui.add_space(8.0);if self.model_present(&MODELS[self.selected]){ui.label("الموديل المحدد موجود بالفعل.");}else if ui.button("تنزيل الموديل المحدد").clicked(){self.start_model_download();}},
   Page::Downloads=>{ui.heading("التنزيلات");ui.label(&self.download_label);if !self.download_speed.is_empty(){ui.label(format!("سرعة التنزيل: {}",self.download_speed));}if self.downloading{ui.add(egui::ProgressBar::new(self.download_progress).show_percentage());if ui.button("إيقاف التنزيل").clicked(){self.cancel.store(true,Ordering::Relaxed);}}else{ui.label("لا توجد تنزيلات نشطة.");ui.label("الملفات الجزئية محفوظة ويمكن استكمالها عند إعادة المحاولة.");}},
   Page::Chat=>{ui.heading("المحادثة المحلية");if self.engine.is_none(){if ui.button("تشغيل الموديل المحلي").clicked(){self.start_engine();}}else if self.engine_ready{ui.label("● المحرك المحلي جاهز");}else{ui.label("جاري تحميل الموديل والتحقق من جاهزية المحرك...");}egui::ScrollArea::vertical().max_height(470.0).stick_to_bottom(true).show(ui,|ui|{egui::Frame::group(ui.style()).show(ui,|ui|{ui.label(if self.busy{"● يكتب الآن..."}else{"الرد"});ui.label(&self.answer);});});ui.horizontal(|ui|{ui.add_enabled(!self.busy,egui::TextEdit::singleline(&mut self.prompt).hint_text("اكتب رسالتك...").desired_width(f32::INFINITY));if self.busy{if ui.button("■ إيقاف").clicked(){self.stop.store(true,Ordering::Relaxed);}}else if ui.add_enabled(self.engine_ready,egui::Button::new("إرسال  ➤")).clicked(){self.send_prompt();}});},
   Page::Tools=>{ui.heading("الموديولات والبرامج المحلية");ui.label("يفحص Doctor AI المسارات الشائعة وPATH؛ الحالات غير المكتشفة يمكن إضافتها يدوياً لاحقاً.");for (name,installed) in &self.installed_tools{ui.horizontal(|ui|{ui.label(name);ui.label(if installed{"موجود محلياً ✓"}else{"غير مكتشف"});});}ui.separator();ui.label("روابط رسمية للتنزيل؛ البرامج التالية اختيارية وليست مدمجة داخل المثبت.");for(n,d,u)in[("ComfyUI","توليد الصور وواجهات Stable Diffusion.","https://github.com/comfyanonymous/ComfyUI"),("Krita","تحرير ورسم الصور الرقمية.","https://krita.org/"),("Whisper.cpp","تحويل الصوت إلى نص محلياً؛ يعمل على CPU ويمكن تشغيل نماذج صغيرة بذاكرة محدودة.","https://github.com/ggml-org/whisper.cpp"),("Piper","تحويل النص إلى صوت محلياً؛ محرك خفيف يعمل على CPU.","https://github.com/rhasspy/piper"),("Blender","نمذجة وتحريك ومونتاج ثلاثي الأبعاد.","https://www.blender.org/download/"),("Shotcut","تحرير فيديو مجاني.","https://shotcut.org/download/"),("Kdenlive","مونتاج متعدد المسارات.","https://kdenlive.org/download/"),("Wan 2.1","نماذج مفتوحة لتوليد الفيديو؛ تتطلب موارد كبيرة.","https://github.com/Wan-Video/Wan2.1")]{ui.horizontal(|ui|{ui.strong(n);ui.label(d);if ui.button("فتح").clicked(){let _=open::that(u);}});}},
   Page::Settings=>{ui.heading("الإعدادات");ui.label("مجلد الموديلات:");ui.code(catalog::model_dir().display().to_string());ui.label(format!("خادم الموديل المحلي: http://127.0.0.1:{}",self.engine_port));
  ui.label(format!("الموديلات المكتشفة محلياً: {}",self.installed_models.len()));ui.label(format!("سجل المحرك: {}",catalog::model_dir().parent().unwrap_or_else(||std::path::Path::new(".")).join("mistralrs.log").display()));if ui.button("إيقاف المحرك").clicked(){if let Some(mut c)=self.engine.take(){let _=c.kill();self.engine_ready=false;self.status="تم إيقاف المحرك".into();}}}
  });
 }
}
fn bytes(n:u64)->String{if n>1024*1024*1024{format!("{:.2} GB",n as f64/1024f64.powi(3))}else{format!("{:.1} MB",n as f64/1024f64.powi(2))}}


fn probe_engine(port:u16)->bool{
 if port==0{return false;}
 let Ok(mut stream)=TcpStream::connect_timeout(&format!("127.0.0.1:{port}").parse().unwrap(),Duration::from_millis(180)) else{return false;};
 let _=stream.set_read_timeout(Some(Duration::from_millis(250)));
 let request=b"GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
 if stream.write_all(request).is_err(){return false;}
 let mut response=[0u8;512];
 match stream.read(&mut response){Ok(n)=>String::from_utf8_lossy(&response[..n]).starts_with("HTTP/1.1 200")||String::from_utf8_lossy(&response[..n]).starts_with("HTTP/1.0 200"),Err(_)=>false}
}

impl Drop for DoctorApp{
 fn drop(&mut self){if let Some(child)=self.engine.as_mut(){let _=child.kill();let _=child.wait();}}
}
