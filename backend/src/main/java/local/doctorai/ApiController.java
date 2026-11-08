package local.doctorai;

import java.net.URI;
import java.nio.file.*;
import java.util.*;
import org.springframework.core.env.Environment;
import org.springframework.http.MediaType;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.client.RestClient;

@RestController
@RequestMapping("/api")
@CrossOrigin(origins = "http://127.0.0.1")
public class ApiController {
    private volatile String ollamaBaseUrl;
    private final Path settingsFile = Path.of(System.getProperty("user.home"), ".doctor-ai", "ollama-url.txt");

    public ApiController(Environment env) {
        String configured = env.getProperty("ollama.base-url", "http://127.0.0.1:11434");
        try {
            if (Files.exists(settingsFile)) configured = Files.readString(settingsFile).trim();
        } catch (Exception ignored) {}
        ollamaBaseUrl = normalizeUrl(configured);
    }

    private RestClient ollama() {
        return RestClient.builder().baseUrl(ollamaBaseUrl).build();
    }

    private String normalizeUrl(String value) {
        String url = value == null ? "" : value.trim().replaceAll("/+$", "");
        URI uri = URI.create(url);
        if (!("http".equalsIgnoreCase(uri.getScheme()) || "https".equalsIgnoreCase(uri.getScheme()))
                || uri.getHost() == null || uri.getUserInfo() != null) {
            throw new IllegalArgumentException("Enter a valid HTTP/HTTPS Ollama address.");
        }
        return url;
    }

    @GetMapping("/settings")
    public Map<String, Object> settings() {
        return Map.of("ollamaUrl", ollamaBaseUrl);
    }

    @PostMapping("/settings")
    public Map<String, Object> updateSettings(@RequestBody Map<String, String> body) {
        try {
            String next = normalizeUrl(body.get("ollamaUrl"));
            Path parent = settingsFile.getParent();
            Files.createDirectories(parent);
            Files.writeString(settingsFile, next, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING);
            ollamaBaseUrl = next;
            return Map.of("ok", true, "ollamaUrl", ollamaBaseUrl, "message", "Settings saved.");
        } catch (Exception e) {
            return Map.of("ok", false, "message", "Could not save settings: " + e.getMessage());
        }
    }

    @GetMapping("/health")
    public Map<String, Object> health() {
        try {
            ollama().get().uri("/api/tags").retrieve().body(Map.class);
            return Map.of("ok", true, "ollama", true, "message", "Ollama connected");
        } catch (Exception e) {
            return Map.of("ok", false, "ollama", false, "message", "Ollama is not reachable: " + e.getMessage());
        }
    }

    @GetMapping("/models")
    public Map<String, Object> models() {
        try {
            Map<?, ?> response = ollama().get().uri("/api/tags").retrieve().body(Map.class);
            List<String> names = new ArrayList<>();
            Object raw = response == null ? null : response.get("models");
            if (raw instanceof List<?> list) {
                for (Object item : list) {
                    if (item instanceof Map<?, ?> model && model.get("name") != null) {
                        names.add(model.get("name").toString());
                    }
                }
            }
            Collections.sort(names);
            String active = names.stream().filter(n -> n.toLowerCase(Locale.ROOT).contains("gemma"))
                    .findFirst().orElse(names.isEmpty() ? "" : names.get(0));
            return Map.of("models", names, "active", active, "count", names.size());
        } catch (Exception e) {
            return Map.of("models", List.of(), "active", "", "message", "Local model search failed: " + e.getMessage());
        }
    }

    @PostMapping("/chat")
    public Map<String, Object> chat(@RequestBody Map<String, String> body) {
        String prompt = body.getOrDefault("message", "").trim();
        if (prompt.isEmpty()) return Map.of("answer", "Please enter a message.");
        String model = body.getOrDefault("model", "");
        if (model.isBlank()) model = String.valueOf(models().get("active"));
        if (model.isBlank()) return Map.of("message", "No Ollama model is installed. Use Search local models or install one with Ollama first.");
        try {
            Map<String, Object> payload = Map.of("model", model, "stream", false,
                    "messages", List.of(Map.of("role", "user", "content", prompt)));
            Map<?, ?> result = ollama().post().uri("/api/chat").contentType(MediaType.APPLICATION_JSON)
                    .body(payload).retrieve().body(Map.class);
            Object message = result == null ? null : result.get("message");
            Object answer = message instanceof Map<?, ?> m ? m.get("content") : "";
            return Map.of("answer", String.valueOf(answer));
        } catch (Exception e) {
            return Map.of("message", "Local model request failed: " + e.getMessage());
        }
    }
}
