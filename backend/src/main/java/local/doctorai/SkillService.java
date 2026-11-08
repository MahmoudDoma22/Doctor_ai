package local.doctorai;

import java.nio.file.*;
import java.io.IOException;
import java.util.*;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

@Service
public class SkillService {
  private final Path dir;
  private static final Set<String> MODULES = Set.of("general", "clinical", "image", "video", "audio");

  public SkillService(@Value("${doctorai.skills-dir:./skills}") String skillsDir) {
    dir = Paths.get(skillsDir).toAbsolutePath().normalize();
    try {
      Files.createDirectories(dir);
      for (String module : MODULES) Files.createDirectories(dir.resolve(module));
    } catch (IOException e) { throw new RuntimeException(e); }
  }

  public List<String> names(String module) {
    Path folder = moduleDir(module);
    try (var stream = Files.list(folder)) {
      return stream.filter(p -> p.toString().endsWith(".md"))
        .map(p -> p.getFileName().toString()).sorted().toList();
    } catch (IOException e) { return List.of(); }
  }

  public String prompt(String module) {
    StringBuilder out = new StringBuilder();
    for (String name : names(module)) {
      try { out.append("\n\n## ").append(module).append(" skill: ").append(name).append("\n")
        .append(Files.readString(moduleDir(module).resolve(name))).append("\n"); }
      catch (IOException ignored) {}
    }
    return out.toString();
  }

  public void save(String module, String name, String content) throws IOException {
    String safe = name == null ? "" : name.replaceAll("[^a-zA-Z0-9_-]", "");
    if (safe.isBlank()) throw new IllegalArgumentException("Invalid skill name");
    Files.writeString(moduleDir(module).resolve(safe + ".md"), content == null ? "" : content);
  }

  private Path moduleDir(String module) {
    String key = module == null ? "general" : module.toLowerCase(Locale.ROOT).trim();
    if (!MODULES.contains(key)) throw new IllegalArgumentException("Unknown skill module: " + key);
    Path folder = dir.resolve(key).normalize();
    if (!folder.startsWith(dir)) throw new IllegalArgumentException("Invalid module path");
    try { Files.createDirectories(folder); } catch (IOException e) { throw new RuntimeException(e); }
    return folder;
  }
}
