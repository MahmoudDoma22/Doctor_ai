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
    try { Map<?,?> tags=ollama.tags(); return Map.of("ok",true,"ollama",true,"model",ollama.model(),"models",tags.get("models") == null ? List.of() : tags.get("models")); }
    catch(Exception e) { return Map.of("ok",false,"ollama",false,"model",ollama.model(),"error",e.getMessage()==null?"Ollama unavailable":e.getMessage()); }
  }
  @GetMapping("/models") public Map<?,?> models() { return ollama.tags(); }

  @PostMapping("/chat")
  public Map<String,String> chat(@RequestBody Map<String,Object> body) throws Exception {
    String message=String.valueOf(body.getOrDefault("message",""));
    List<String> images = new ArrayList<>();
    Object raw=body.get("images");
    if(raw instanceof List<?> list) for(Object item:list) if(item instanceof String s && !s.isBlank()) images.add(s);
    String system="You are Doctor AI, a concise, accurate, helpful assistant. Be direct and to the point. "
      +"For medical questions, distinguish evidence from uncertainty, encourage appropriate clinical judgment, "
      +"and never claim to replace a qualified clinician. Prefer structured concise answers."
      + skills.prompt();
    // Video is sampled into still frames by the local Python media service.
    Object rawVideos = body.get("videos");
    if (rawVideos instanceof List<?> videos) {
      RestClient media = RestClient.builder().baseUrl(mediaBase).build();
      for (Object item : videos) {
        if (!(item instanceof String video64) || video64.isBlank()) continue;
        try {
          Map<?,?> extracted = media.post().uri("/extract-frames")
            .contentType(MediaType.APPLICATION_JSON)
            .body(Map.of("video_base64", video64, "max_frames", 8))
            .retrieve().body(Map.class);
          Object frameList = extracted == null ? null : extracted.get("frames");
          if (frameList instanceof List<?> frames) {
            for (Object frame : frames) if (frame instanceof String frame64) images.add(frame64);
          }
        } catch (Exception mediaError) {
          throw new IllegalStateException("Video processing failed. Is the Python media service running? " + mediaError.getMessage(), mediaError);
        }
      }
    }
    String answer=ollama.chat(message,images,system);
    saveTurn("user",message.isBlank()?"[image attachment]":message);
    saveTurn("assistant",answer);
    return Map.of("answer",answer,"model",ollama.model());
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
  @GetMapping("/skills") public Map<String,Object> getSkills() { return Map.of("skills",skills.names()); }
  @PostMapping("/skills")
  public Map<String,String> saveSkill(@RequestBody Map<String,String> body) throws Exception {
    skills.save(body.getOrDefault("name","new-skill"),body.getOrDefault("content",""));
    return Map.of("status","saved");
  }
}
