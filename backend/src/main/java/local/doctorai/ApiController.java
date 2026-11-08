package local.doctorai;

import java.util.*;
import java.nio.file.*;
import java.sql.*;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.http.*;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.client.RestClient;

@RestController
@RequestMapping("/api")
public class ApiController {
  private final OllamaService ollama;
  private final SkillService skills;
  private final Path dbPath;
  private final String mediaBase;

  public ApiController(OllamaService ollama, SkillService skills,
      @Value("${doctorai.data-dir}") String dataDir,
      @Value("${media.base-url}") String mediaBase) throws Exception {
    this.ollama=ollama; this.skills=skills; this.mediaBase=mediaBase;
    Path dir=Paths.get(dataDir); Files.createDirectories(dir); dbPath=dir.resolve("doctorai.sqlite");
    try (Connection c=connection(); Statement s=c.createStatement()) {
      s.execute("CREATE TABLE IF NOT EXISTS turns(id INTEGER PRIMARY KEY AUTOINCREMENT, role TEXT, content TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP)");
    }
  }
  private Connection connection() throws SQLException { return DriverManager.getConnection("jdbc:sqlite:"+dbPath); }

  @GetMapping("/health")
  public Map<String,Object> health() {
    try {
      List<String> models=ollama.modelNames();
      String active=ollama.model();
      if (active.isBlank() && !models.isEmpty()) active=ollama.selectModel(models.stream().filter(n -> n.toLowerCase(Locale.ROOT).startsWith("gemma")).findFirst().orElse(models.get(0)));
      return Map.of("ok",!models.isEmpty(),"ollama",true,"model",active,"models",models);
    } catch(Exception e) {
      return Map.of("ok",false,"ollama",false,"model",ollama.model(),"models",List.of(),"error",e.getMessage()==null?"Ollama unavailable":e.getMessage());
    }
  }

  @GetMapping("/models")
  public Map<String,Object> models() {
    List<String> names=ollama.modelNames();
    String active=ollama.model();
    if ((active.isBlank() || !names.contains(active)) && !names.isEmpty())
      active=ollama.selectModel(names.stream().filter(n -> n.toLowerCase(Locale.ROOT).startsWith("gemma")).findFirst().orElse(names.get(0)));
    return Map.of("models",names,"activeModel",active);
  }

  @PostMapping("/models/select")
  public Map<String,String> selectModel(@RequestBody Map<String,String> body) {
    return Map.of("activeModel",ollama.selectModel(body.get("model")));
  }

  @PostMapping("/chat")
  public Map<String,String> chat(@RequestBody Map<String,Object> body) throws Exception {
    String message=String.valueOf(body.getOrDefault("message",""));
    String module=String.valueOf(body.getOrDefault("module","general")).toLowerCase(Locale.ROOT);
    List<String> images = new ArrayList<>();
    Object raw=body.get("images");
    if(raw instanceof List<?> list) for(Object item:list) if(item instanceof String s && !s.isBlank()) images.add(s);
    Object rawVideos = body.get("videos");
    if (rawVideos instanceof List<?> videos) {
      RestClient media = RestClient.builder().baseUrl(mediaBase).build();
      for (Object item : videos) {
        if (!(item instanceof String video64) || video64.isBlank()) continue;
        Map<?,?> extracted = media.post().uri("/extract-frames").contentType(MediaType.APPLICATION_JSON)
          .body(Map.of("video_base64", video64, "max_frames", 8)).retrieve().body(Map.class);
        Object frameList = extracted == null ? null : extracted.get("frames");
        if (frameList instanceof List<?> frames)
          for (Object frame : frames) if (frame instanceof String frame64) images.add(frame64);
      }
    }
    String system="You are Doctor AI, a concise, accurate, helpful assistant. Be direct. "
      +"For medical questions, distinguish evidence from uncertainty, encourage appropriate clinical judgment, "
      +"and never claim to replace a qualified clinician. Prefer structured concise answers."
      + skills.prompt(module);
    String answer=ollama.chat(message,images,system);
    saveTurn("user",message.isBlank()?"[media attachment]":message);
    saveTurn("assistant",answer);
    return Map.of("answer",answer,"model",ollama.model(),"module",module);
  }

  private void saveTurn(String role,String content) {
    try(Connection c=connection(); PreparedStatement p=c.prepareStatement("INSERT INTO turns(role,content) VALUES(?,?)")) {
      p.setString(1,role); p.setString(2,content); p.executeUpdate();
    } catch(Exception ignored) {}
  }

  @GetMapping("/history")
  public List<Map<String,Object>> history() throws Exception {
    List<Map<String,Object>> out=new ArrayList<>();
    try(Connection c=connection(); Statement s=c.createStatement(); ResultSet r=s.executeQuery("SELECT id,role,content,created_at FROM turns ORDER BY id DESC LIMIT 100")) {
      while(r.next()) out.add(Map.of("id",r.getInt("id"),"role",r.getString("role"),"content",r.getString("content"),"createdAt",r.getString("created_at")));
    }
    return out;
  }
  @GetMapping("/skills")
  public Map<String,Object> getSkills(@RequestParam(defaultValue="general") String module) { return Map.of("module",module,"skills",skills.names(module)); }

  @PostMapping("/skills")
  public Map<String,String> saveSkill(@RequestBody Map<String,String> body) throws Exception {
    String module=body.getOrDefault("module","general");
    skills.save(module,body.getOrDefault("name","new-skill"),body.getOrDefault("content",""));
    return Map.of("status","saved","module",module);
  }
}
