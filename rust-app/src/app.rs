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
 scan_rx:Option<std::sync::mpsc::Receiver<(Vec<std::path::PathBuf>,Vec<(&'static str,bool)>)>>,scan_complete:bool, 
}
impl DoctorApp{
 pub fn new()->Self{
  let(tx,rx)=mpsc::unbounded_channel();let ram=catalog::detect_ram_gb();let available_ram=catalog::detect_available_ram_gb();let free_disk=catalog::detect_free_disk_gb();let cpu=catalog::detect_cpu();
  let mut app=Self{page:Page::Chat,ram,available_ram,free_disk,cpu,selected:catalog::recommended(ram,available_ram),notices:vec!["Welcome to Doctor AI".into()],download_rx:rx,download_tx:tx,cancel:Arc::new(AtomicBool::new(false)),downloading:false,download_progress:0.0,download_label:String::new(),prompt:String::new(),answer:String::new(),busy:false,stop:Arc::new(AtomicBool::new(false)),engine:None,status:"Ready".into(),chat_rx:None,download_speed:"".into(),last_progress:None,http:reqwest::Client::builder().pool_max_idle_per_host(4).connect_timeout(Duration::from_secs(5)).build().unwrap_or_default(),engine_port:0,engine_ready:false,engine_started_at:None,installed_models:Vec::new(),installed_tools:Vec::new(),scan_rx:None,scan_complete:false};
  app.begin_scan();
  app
 }
 fn begin_scan(&mut self){
  let(tx,rx)=std::sync::mpsc::channel();
  self.scan_rx=Some(rx);self.scan_complete=false;self.status="Scanning local models and tools...".into();
  std::thread::spawn(move||{let models=catalog::installed_model_files();let tools=catalog::installed_tools();let _=tx.send((models,tools));});
 }
 fn model_present(&self,m:&catalog::Model)->bool{
  self.installed_models.iter().any(|p|p.file_name().and_then(|n|n.to_str()).map(|n|n.eq_ignore_ascii_case(m.file_name)).unwrap_or(false))
 }
 fn start_model_download(&mut self){
  let required_disk=if self.selected==1{6}else{4};
  let free_disk=catalog::detect_free_disk_gb();
  self.free_disk=free_disk;
  if free_disk<required_disk{self.status=format!("Insufficient disk space: available {} GB required approximately {} GB on the model folder drive",free_disk,required_disk);return;}
  let m=&MODELS[self.selected];self.cancel.store(false,Ordering::Relaxed);self.downloading=true;self.download_progress=0.0;
  self.download_label=format!("Downloading {}",m.name);self.status="Downloading model — resume is supported after restarting the app".into();
  download::start_download(m.url.into(),catalog::model_path(m),self.cancel.clone(),self.download_tx.clone());
 }
 fn start_engine(&mut self){
  let Some(path)=catalog::find_model_path(&MODELS[self.selected]) else {self.status="Model not found. Open Models to download it.".into();return;};
  let exe=std::env::current_exe().ok().and_then(|p|p.parent().map(|p|p.join("runtime").join("mistralrs").join("mistralrs.exe")));
  let Some(exe)=exe.filter(|p|p.exists())else{self.status="Rust engine is missing from runtime/mistralrs".into();return;};
  let log_path=catalog::model_dir().parent().unwrap_or_else(||std::path::Path::new(".")).join("mistralrs.log");
  let listener=match TcpListener::bind(("127.0.0.1",0)){Ok(l)=>l,Err(e)=>{self.status=format!("Could not reserve a local engine port: {e}");return;}};
  self.engine_port=listener.local_addr().map(|a|a.port()).unwrap_or(0);drop(listener);
  if self.engine_port==0{self.status="Could not determine a local engine port".into();return;}
  if let Some(parent)=log_path.parent(){if let Err(e)=std::fs::create_dir_all(parent){self.status=format!("Could not create log folder {}: {e}",parent.display());return;}}
  let log=match std::fs::OpenOptions::new().create(true).append(true).open(&log_path){Ok(f)=>f,Err(e)=>{self.status=format!("Could not open engine log {}: {e}",log_path.display());return;}};
  let stdout=match log.try_clone(){Ok(f)=>Stdio::from(f),Err(e)=>{self.status=format!("Could not prepare engine log {}: {e}",log_path.display());return;}};
  let mut command=Command::new(exe);command.args(["serve","-f"]).arg(path).args(["--host","127.0.0.1","--port",&self.engine_port.to_string(),"--no-ui"]).stdout(stdout).stderr(Stdio::from(log));#[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}match command.spawn(){Ok(c)=>{self.engine=Some(c);self.engine_ready=false;self.engine_started_at=Some(Instant::now());self.status=format!("Loading the model and checking engine readiness on port {}; log: {}",self.engine_port,log_path.display());},Err(e)=>self.status=format!("Failed to start engine; log: {} — {e}",log_path.display())}
 }
 fn send_prompt(&mut self){
  if self.prompt.trim().is_empty()||self.busy{return;}
  if !self.engine_ready{self.status="The engine is not ready yet. Wait for the model to finish loading.".into();return;}
  let prompt=std::mem::take(&mut self.prompt);
  self.answer.clear();self.busy=true;self.stop.store(false,Ordering::Relaxed);self.status="Connecting to the local model...".into();
  let stop=self.stop.clone();let http=self.http.clone();let port=self.engine_port;let(tx,rx)=std::sync::mpsc::channel();let tx_error=tx.clone();self.chat_rx=Some(rx);
  std::thread::spawn(move||{
   let rt=tokio::runtime::Builder::new_multi_thread().enable_all().build();
   let result=match rt{Ok(rt)=>rt.block_on(async move{
    let body=json!({"model":"default","stream":true,"messages":[{"role":"user","content":prompt}]});
    let mut response=http.post(format!("http://127.0.0.1:{port}/v1/chat/completions")).json(&body).send().await.map_err(|e|format!("Local engine connection: {e}"))?;
    if !response.status().is_success(){return Err(format!("Engine returned HTTP {}",response.status()));}
    let mut buffer:Vec<u8>=Vec::new();
    loop {
     let next = tokio::select! {
      _ = tokio::time::sleep(Duration::from_millis(100)) => {
       if stop.load(Ordering::Relaxed) { break; }
       continue;
      }
      chunk = response.chunk() => chunk.map_err(|e|format!("Receiving data: {e}"))?,
     };
     let Some(chunk)=next else { break; };
     buffer.extend_from_slice(&chunk);
     while let Some(pos)=buffer.iter().position(|b|*b==b'\n'){let line=String::from_utf8_lossy(&buffer[..pos]).trim().to_string();buffer.drain(..=pos);
      if let Some(data)=line.strip_prefix("data:").map(str::trim){if data=="[DONE]"{return Ok::<(),String>(());}if let Ok(v)=serde_json::from_str::<serde_json::Value>(data){if let Some(s)=v["choices"][0]["delta"]["content"].as_str(){let _=tx.send(s.to_string());}}}
     }
    }Ok::<(),String>(())
   }),Err(e)=>Err(format!("Tokio initialization: {e}"))};
   if let Err(e)=result{let _=tx_error.send(format!("\n[Error] {e}"));}
  });
 }
 fn poll(&mut self){
  let scan_result=self.scan_rx.as_ref().and_then(|rx|rx.try_recv().ok());
  if let Some((models,tools))=scan_result{self.installed_models=models;self.installed_tools=tools;self.scan_complete=true;self.scan_rx=None;self.status="Local model and tool scan completed".into();}
  while let Ok(ev)=self.download_rx.try_recv(){match ev{
   DownloadEvent::Progress(n,total)=>{let now=std::time::Instant::now();if let Some((old,at))=self.last_progress{let secs=now.duration_since(at).as_secs_f64().max(0.001);self.download_speed=format!("{}/s",bytes(((n.saturating_sub(old))as f64/secs)as u64));}self.last_progress=Some((n,now));self.download_progress=total.map(|t|(n as f32/t.max(1)as f32).clamp(0.0,1.0)).unwrap_or(0.0);self.download_label=format!("{} / {}",bytes(n),total.map(bytes).unwrap_or_else(||"Unknown size".into()));}
   DownloadEvent::Finished(p)=>{self.downloading=false;if !self.installed_models.contains(&p){self.installed_models.push(p.clone());}self.status="Download completed".into();self.notices.push(format!("Downloaded {}",p.display()));}
   DownloadEvent::Error(e)=>{self.downloading=false;self.status=format!("Error: {e}");self.notices.push(self.status.clone());}
   DownloadEvent::Cancelled=>{self.downloading=false;self.status="Download stopped; partial file saved".into();}
  }}
  let child_exit=self.engine.as_mut().map(|child|child.try_wait());
  match child_exit{Some(Ok(Some(code)))=>{self.engine_ready=false;self.engine=None;self.status=format!("Rust engine exited with code {code}. See mistralrs.log in the .doctor-ai folder");},Some(Err(e))=>{self.engine_ready=false;self.status=format!("Could not check engine process: {e}");},Some(Ok(None))=>{let ready=probe_engine(self.engine_port);if ready&&!self.engine_ready{self.status="Local engine ready — you can start chatting".into();}self.engine_ready=ready;if !ready{if let Some(start)=self.engine_started_at{if start.elapsed()>Duration::from_secs(180){self.status="Model loading took over 3 minutes. Check mistralrs.log and available memory.".into();}}}},None=>{self.engine_ready=false;}}
  let mut disconnected=false;if let Some(rx)=&self.chat_rx{loop{match rx.try_recv(){Ok(s)=>self.answer.push_str(&s),Err(std::sync::mpsc::TryRecvError::Empty)=>break,Err(std::sync::mpsc::TryRecvError::Disconnected)=>{disconnected=true;break;}}}}
  if disconnected{self.chat_rx=None;self.busy=false;if !self.stop.load(Ordering::Relaxed){self.status="Response complete".into();}}
  if self.stop.load(Ordering::Relaxed)&&self.busy{self.busy=false;self.chat_rx=None;self.status="Response stopped".into();}
 }
 fn nav(&mut self,ui:&mut egui::Ui,p:Page,label:&str){if ui.selectable_label(self.page==p,label).clicked(){self.page=p;}}
}
impl eframe::App for DoctorApp{
 fn update(&mut self,ctx:&egui::Context,_:&mut eframe::Frame){
  self.poll();ctx.request_repaint_after(Duration::from_millis(80));
  egui::TopBottomPanel::top("header").show(ctx,|ui|{ui.horizontal(|ui|{ui.heading("✚ Doctor AI");ui.separator();ui.label("Private local AI");});});
  egui::SidePanel::left("nav").resizable(false).default_width(190.0).show(ctx,|ui|{ui.add_space(12.0);ui.heading("Navigation");ui.separator();self.nav(ui,Page::Welcome,"⌂  Home");self.nav(ui,Page::Chat,"✧  Chat");self.nav(ui,Page::Models,"▣  Models");self.nav(ui,Page::Downloads,"⇩  Downloads");self.nav(ui,Page::Tools,"▤  Media tools");self.nav(ui,Page::Settings,"⚙  Settings");ui.with_layout(egui::Layout::bottom_up(egui::Align::Min),|ui|{ui.separator();ui.label("Notifications");for n in self.notices.iter().rev().take(4){ui.label(egui::RichText::new(n).small().weak());}});});
  egui::TopBottomPanel::bottom("status").show(ctx,|ui|{ui.horizontal(|ui|{ui.label("●");ui.label(&self.status);});});
  egui::CentralPanel::default().show(ctx,|ui|match self.page{
   Page::Welcome=>{ui.add_space(24.0);egui::Frame::group(ui.style()).show(ui,|ui|{ui.horizontal(|ui|{ui.heading(egui::RichText::new("✚").size(48.0).color(egui::Color32::from_rgb(56,211,190)));ui.vertical(|ui|{ui.heading(egui::RichText::new("DOCTOR AI").size(30.0).strong());ui.label("Your local AI assistant — powered by Rust and designed for privacy");});});ui.add_space(10.0);ui.label("Native Rust app • Local inference • Model control • Privacy");});ui.add_space(16.0);ui.heading("Welcome — let's set up your device");ui.label("Automatic hardware check, installed model discovery, and a suitable model recommendation.");ui.add_space(10.0);egui::Frame::group(ui.style()).show(ui,|ui|{ui.heading("Device check");ui.columns(2,|cols|{cols[0].label(format!("Total memory: {} GB",self.ram));cols[0].label(format!("Available memory: {} GB",self.available_ram));cols[1].label(format!("Free disk space: {} GB",self.free_disk));cols[1].label(format!("Processor: {}",self.cpu));});ui.separator();let m=&MODELS[catalog::recommended(self.ram,self.available_ram)];ui.strong(format!("Recommended model: {}",m.name));ui.label(m.description);ui.label(if self.model_present(m){"Model found in known local folders ✓"}else{"Model not found — download with resume support"});});ui.add_space(12.0);ui.horizontal(|ui|{if ui.button("1 — Choose and download a model").clicked(){self.page=Page::Models;}if ui.button("2 — Open Chat").clicked(){self.page=Page::Chat;}});ui.add_space(8.0);ui.label(if self.scan_complete{format!("Detected {} GGUF model files and {} local tools.",self.installed_models.len(),self.installed_tools.iter().filter(|(_,v)|*v).count())}else{"Scanning local models and tools...".into()});},
   Page::Models=>{ui.heading("Model library");ui.label(format!("Device: {} GB RAM • {}",self.ram,self.cpu));ui.label(format!("GGUF files found in known folders: {}",self.installed_models.len()));for path in self.installed_models.iter(){ui.label(format!("Local: {}",path.display()));}for(i,m)in MODELS.iter().enumerate(){egui::Frame::group(ui.style()).show(ui,|ui|{ui.horizontal(|ui|{ui.radio_value(&mut self.selected,i,"");ui.vertical(|ui|{ui.strong(m.name);ui.label(m.description);ui.label(format!("Recommended RAM: {} GB • Size: {} • {}",m.ram_gb,m.size,m.tier));});ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if self.model_present(m){ui.label("Installed ✓");}else if ui.button("Open model source").clicked(){let _=open::that(m.url);}});});});}ui.add_space(8.0);if self.model_present(&MODELS[self.selected]){ui.label("Selected model is already installed.");}else if ui.add_enabled(self.scan_complete&&!self.downloading,egui::Button::new("Download selected model")).clicked(){self.start_model_download();}},
   Page::Downloads=>{ui.heading("Downloads");ui.label(&self.download_label);if !self.download_speed.is_empty(){ui.label(format!("Download speed: {}",self.download_speed));}if self.downloading{ui.add(egui::ProgressBar::new(self.download_progress).show_percentage());if ui.button("Stop download").clicked(){self.cancel.store(true,Ordering::Relaxed);}}else{ui.label("No active downloads.");ui.label("Partial downloads are saved and can resume when retried.");}},
   Page::Chat=>{ui.heading("Chat");if self.engine.is_none(){if ui.button("Start local model").clicked(){self.start_engine();}}else if self.engine_ready{ui.label("● Local engine ready");}else{ui.label("Loading model and checking engine readiness...");}egui::ScrollArea::vertical().max_height(470.0).stick_to_bottom(true).show(ui,|ui|{egui::Frame::group(ui.style()).show(ui,|ui|{ui.label(if self.busy{"● Generating..."}else{"Response"});ui.label(&self.answer);});});ui.horizontal(|ui|{ui.add_enabled(!self.busy,egui::TextEdit::singleline(&mut self.prompt).hint_text("Message Doctor AI...").desired_width(f32::INFINITY));if self.busy{if ui.button("■ Stop").clicked(){self.stop.store(true,Ordering::Relaxed);}}else if ui.add_enabled(self.engine_ready,egui::Button::new("Send  ➤")).clicked(){self.send_prompt();}});},
   Page::Tools=>{ui.heading("Local modules and tools");ui.label("Doctor AI checks common folders and PATH. Undetected tools can be added manually later.");if ui.button("Rescan tools and models").clicked(){self.begin_scan();}if !self.scan_complete{ui.label("Scanning in the background...");}for (name,installed) in &self.installed_tools{ui.horizontal(|ui|{ui.label(*name);ui.label(if *installed{"Available locally ✓"}else{"Not detected"});});}ui.separator();ui.label("Official download links. These optional tools are not bundled with the installer.");for(n,d,u)in[("ComfyUI","Image generation and Stable Diffusion workflows.","https://github.com/comfyanonymous/ComfyUI"),("Krita","Digital painting and image editing.","https://krita.org/"),("Whisper.cpp","Local speech-to-text. Runs on CPU, including smaller models on limited memory.","https://github.com/ggml-org/whisper.cpp"),("Piper","Local text-to-speech with a lightweight CPU engine.","https://github.com/rhasspy/piper"),("Blender","3D modeling, animation, and rendering.","https://www.blender.org/download/"),("Shotcut","Free video editor.","https://shotcut.org/download/"),("Kdenlive","Multitrack video editor.","https://kdenlive.org/download/"),("Wan 2.1","Open video-generation models; requires substantial hardware.","https://github.com/Wan-Video/Wan2.1")]{ui.horizontal(|ui|{ui.strong(n);ui.label(d);if ui.button("Open").clicked(){let _=open::that(u);}});}},
   Page::Settings=>{ui.heading("Settings");ui.label("Model folder:");ui.code(catalog::model_dir().display().to_string());ui.label(format!("Local model server: http://127.0.0.1:{}",self.engine_port));
  ui.label(format!("Locally detected models: {}",self.installed_models.len()));ui.label(format!("Engine log: {}",catalog::model_dir().parent().unwrap_or_else(||std::path::Path::new(".")).join("mistralrs.log").display()));if ui.button("Stop engine").clicked(){if let Some(mut c)=self.engine.take(){let _=c.kill();self.engine_ready=false;self.status="Engine stopped".into();}}}
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
