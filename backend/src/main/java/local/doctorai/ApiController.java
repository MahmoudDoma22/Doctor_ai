package local.doctorai;
import java.lang.management.ManagementFactory;
import java.nio.file.*;
import java.time.Duration;
import java.util.*;
import jakarta.annotation.PreDestroy;
import org.springframework.http.MediaType;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.client.RestClient;

@RestController @RequestMapping("/api") @CrossOrigin(origins="http://127.0.0.1")
public class ApiController {
 private final Path root=Path.of(System.getProperty("user.dir")).toAbsolutePath().normalize();
 private final Path modelDir=Path.of(System.getProperty("user.home"),".doctor-ai","models");
 private final Path log=Path.of(System.getProperty("user.home"),".doctor-ai","llama-server.log");
 private final Path exe=root.resolve("runtime/llama/llama-server.exe");
 private volatile Process process;
 private final RestClient engine=RestClient.builder().baseUrl("http://127.0.0.1:8080").requestFactory(f->f.setConnectTimeout(Duration.ofSeconds(3))).build();

 private Path model(){
  Path e4b=modelDir.resolve("gemma-4-E4B-it-Q4_0.gguf");
  if(Files.isRegularFile(e4b)) return e4b;
  Path e2b=modelDir.resolve("gemma-4-E2B-it-Q4_0.gguf");
  return Files.isRegularFile(e2b)?e2b:null;
 }
 private Path projector(Path m){
  if(m==null)return null;
  String n=m.getFileName().toString().contains("E4B")?"mmproj-gemma-4-E4B-it-BF16.gguf":"mmproj-gemma-4-E2B-it-BF16.gguf";
  Path p=modelDir.resolve(n); return Files.isRegularFile(p)?p:null;
 }
 private synchronized void startEngine(){
  if(process!=null&&process.isAlive())return;
  Path m=model(); if(m==null||!Files.isRegularFile(exe))return;
  try{
   Files.createDirectories(log.getParent());
   int threads=Math.max(2,Math.min(12,Runtime.getRuntime().availableProcessors()-1));
   List<String> args=new ArrayList<>(List.of(exe.toString(),"--model",m.toString(),"--host","127.0.0.1","--port","8080","--ctx-size","2048","--threads",String.valueOf(threads),"--n-gpu-layers","0"));
   Path p=projector(m); if(p!=null){args.add("--mmproj");args.add(p.toString());}
   ProcessBuilder pb=new ProcessBuilder(args);pb.directory(exe.getParent().toFile());pb.redirectErrorStream(true);pb.redirectOutput(ProcessBuilder.Redirect.appendTo(log.toFile()));process=pb.start();
  }catch(Exception e){try{Files.createDirectories(log.getParent());Files.writeString(log,e+System.lineSeparator(),StandardOpenOption.CREATE,StandardOpenOption.APPEND);}catch(Exception ignored){}}
 }
 private boolean ready(){
  if(model()==null||!Files.isRegularFile(exe))return false;
  startEngine();try{engine.get().uri("/health").retrieve().toBodilessEntity();return true;}catch(Exception e){return false;}
 }
 @GetMapping("/health") public Map<String,Object> health(){
  boolean installed=model()!=null,binary=Files.isRegularFile(exe),r=installed&&binary&&ready();
  String msg=!binary?"Bundled local AI engine is missing.":!installed?"No offline model is installed. Run the Doctor AI model setup.":r?"Offline AI engine is ready.":"The model is installed and the local engine is starting. Check llama-server.log if this persists.";
  return Map.of("ok",true,"backend",true,"ready",r,"modelInstalled",installed,"engineAvailable",binary,"engine","llama.cpp","message",msg);
 }
 @GetMapping("/models") public Map<String,Object> models(){
  Path m=model();String n=m==null?"":m.getFileName().toString();
  return Map.of("models",m==null?List.of():List.of(n),"active",n,"count",m==null?0:1,"engine","llama.cpp");
 }
 @GetMapping("/hardware") public Map<String,Object> hardware(){
  long bytes=0;var b=ManagementFactory.getOperatingSystemMXBean();
  if(b instanceof com.sun.management.OperatingSystemMXBean o)bytes=o.getTotalMemorySize();
  long gb=bytes==0?0:Math.round((double)bytes/(1024L*1024L*1024L));int cores=Runtime.getRuntime().availableProcessors();
  String tier=gb<8?"Below offline AI minimum":gb<16?"Entry local AI":gb<32?"Balanced local AI":"High-memory local AI";
  return Map.of("ramGB",gb,"logicalCores",cores,"architecture",System.getProperty("os.arch"),"operatingSystem",System.getProperty("os.name"),"recommendedTier",tier,"gpuAcceleration","CPU backend in this release; GPU backends are separate optional modules.");
 }
 @GetMapping("/modules") public List<Map<String,Object>> modules(){
  return List.of(
   Map.of("id","local-chat","name","Offline chat and medical assistant","category","Language model","description","Gemma 4 E2B/E4B runs entirely on this PC. E2B is lighter; E4B is stronger but needs more memory.","minimum","8 GB RAM for E2B; 16 GB recommended for E4B","status","available"),
   Map.of("id","vision","name","Image understanding","category","Vision","description","Optional Gemma 4 projector for local image analysis.","minimum","8 GB RAM for E2B; 16 GB recommended for E4B","status","available-optional-download"),
   Map.of("id","speech-to-text","name","Speech recognition","category","Audio","description","Whisper.cpp can transcribe audio locally.","minimum","8 GB RAM recommended","status","catalog-only"),
   Map.of("id","text-to-speech","name","Text to speech","category","Audio","description","Piper provides lightweight offline speech synthesis.","minimum","4 GB RAM; CPU","status","catalog-only"),
   Map.of("id","image-generation","name","Image generation","category","Creative","description","Stable Diffusion through ComfyUI for image generation and editing.","minimum","16 GB RAM and 8 GB dedicated VRAM recommended","status","catalog-only"),
   Map.of("id","video-generation","name","Video generation","category","Creative","description","Wan 2.x workflows through ComfyUI; very large downloads.","minimum","32 GB RAM and 16–24 GB dedicated VRAM recommended","status","catalog-only"),
   Map.of("id","audio-generation","name","Audio generation","category","Creative","description","Offline music and sound generation models.","minimum","32 GB RAM and a capable GPU recommended","status","catalog-only")
  );
 }
 @PostMapping("/chat") public Map<String,Object> chat(@RequestBody Map<String,Object> body){
  String prompt=String.valueOf(body.getOrDefault("message","")).trim();Object raw=body.get("images");
  if(prompt.isEmpty()&&(!(raw instanceof List<?> imgs)||imgs.isEmpty()))return Map.of("message","Enter a message or attach an image.");
  Path m=model();if(m==null)return Map.of("message","No offline model is installed. Re-run the installer and select the recommended Gemma 4 model.");
  if(!ready())return Map.of("message","The local AI engine is not ready. Check available RAM and "+log);
  String mode=String.valueOf(body.getOrDefault("performanceMode","balanced")).toLowerCase(Locale.ROOT);
  int tokens=switch(mode){case "fast"->256;case "quality"->768;default->512;};
  Map<String,Object> user=new HashMap<>();user.put("role","user");
  if(raw instanceof List<?> imgs&&!imgs.isEmpty()){
   if(projector(m)==null)return Map.of("message","Image understanding is not installed. Re-run setup and enable the optional vision module.");
   List<Map<String,Object>> content=new ArrayList<>();if(!prompt.isBlank())content.add(Map.of("type","text","text",prompt));
   for(Object img:imgs)content.add(Map.of("type","image_url","image_url",Map.of("url","data:image/jpeg;base64,"+img)));
   user.put("content",content);
  }else user.put("content",prompt);
  try{
   Map<String,Object> payload=Map.of("model",m.getFileName().toString(),"messages",List.of(user),"temperature",0.4,"max_tokens",tokens,"stream",false);
   Map<?,?> result=engine.post().uri("/v1/chat/completions").contentType(MediaType.APPLICATION_JSON).body(payload).retrieve().body(Map.class);
   Object choices=result==null?null:result.get("choices");
   if(choices instanceof List<?> list&&!list.isEmpty()&&list.get(0) instanceof Map<?,?> c&&c.get("message") instanceof Map<?,?> msg){
    Object answer=msg.get("content");return Map.of("answer",answer==null?"":answer.toString(),"model",m.getFileName().toString(),"performanceMode",mode);
   }
   return Map.of("message","The local model returned an empty response.");
  }catch(Exception e){return Map.of("message","Local inference failed: "+e.getMessage()+". Check "+log);}
 }
 @PreDestroy public synchronized void stopEngine(){if(process!=null&&process.isAlive())process.destroy();}
}