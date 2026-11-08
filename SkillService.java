package local.doctorai;
import java.nio.file.*;
import java.io.IOException;
import java.util.*;
import org.springframework.stereotype.Service;

@Service
public class SkillService {
  private final Path dir = Paths.get("skills").toAbsolutePath().normalize();
  public SkillService() { try { Files.createDirectories(dir); } catch(IOException e) { throw new RuntimeException(e); } }
  public List<String> names() {
    try (var stream = Files.list(dir)) {
      return stream.filter(p -> p.toString().endsWith(".md")).map(p -> p.getFileName().toString()).sorted().toList();
    } catch(IOException e) { return List.of(); }
  }
  public String prompt() {
    StringBuilder out = new StringBuilder();
    for (String name : names()) {
      try { out.append("\n\n## Skill: ").append(name).append("\n")
        .append(Files.readString(dir.resolve(name))).append("\n"); }
      catch(IOException ignored) {}
    }
    return out.toString();
  }
  public void save(String name, String content) throws IOException {
    String safe = name.replaceAll("[^a-zA-Z0-9_-]", "");
    if (safe.isBlank()) throw new IllegalArgumentException("Invalid skill name");
    Files.writeString(dir.resolve(safe + ".md"), content == null ? "" : content);
  }
}
