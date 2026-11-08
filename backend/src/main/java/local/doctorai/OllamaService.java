package local.doctorai;

import java.util.*;
import java.util.concurrent.atomic.AtomicReference;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.http.MediaType;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestClient;

@Service
public class OllamaService {
  private final RestClient client;
  private final AtomicReference<String> activeModel;

  public OllamaService(@Value("${ollama.base-url}") String baseUrl,
                       @Value("${ollama.model:}") String configuredModel) {
    this.client = RestClient.builder().baseUrl(baseUrl).build();
    this.activeModel = new AtomicReference<>(configuredModel == null ? "" : configuredModel.trim());
  }

  public String model() { return activeModel.get(); }

  @SuppressWarnings("unchecked")
  public Map<String,Object> tags() {
    Map<String,Object> result = client.get().uri("/api/tags").retrieve().body(Map.class);
    return result == null ? Map.of("models", List.of()) : result;
  }

  public List<String> modelNames() {
    Object raw = tags().get("models");
    if (!(raw instanceof List<?> list)) return List.of();
    List<String> names = new ArrayList<>();
    for (Object item : list) {
      if (item instanceof Map<?,?> model && model.get("name") != null)
        names.add(String.valueOf(model.get("name")));
    }
    Collections.sort(names, String.CASE_INSENSITIVE_ORDER);
    return names;
  }

  public String selectModel(String requested) {
    if (requested == null || requested.isBlank()) throw new IllegalArgumentException("Model name is required.");
    String match = modelNames().stream().filter(n -> n.equals(requested.trim())).findFirst()
      .orElseThrow(() -> new IllegalArgumentException("This model is not installed in Ollama: " + requested));
    activeModel.set(match);
    return match;
  }

  private String ensureModel() {
    String current = activeModel.get();
    List<String> names = modelNames();
    if (!current.isBlank() && names.contains(current)) return current;
    if (names.isEmpty()) throw new IllegalStateException("No local Ollama models are installed. Install a model and refresh the list.");
    String preferred = names.stream().filter(n -> n.toLowerCase(Locale.ROOT).startsWith("gemma")).findFirst().orElse(names.get(0));
    activeModel.set(preferred);
    return preferred;
  }

  public String chat(String prompt, List<String> images, String system) {
    String selected = ensureModel();
    Map<String,Object> user = new LinkedHashMap<>();
    user.put("role", "user");
    user.put("content", prompt == null ? "" : prompt);
    if (images != null && !images.isEmpty()) user.put("images", images);
    List<Map<String,Object>> messages = new ArrayList<>();
    messages.add(Map.of("role", "system", "content", system));
    messages.add(user);
    Map<String,Object> body = new LinkedHashMap<>();
    body.put("model", selected);
    body.put("messages", messages);
    body.put("stream", false);
    body.put("keep_alive", "10m");
    Map<?,?> result = client.post().uri("/api/chat")
      .contentType(MediaType.APPLICATION_JSON).body(body).retrieve().body(Map.class);
    Object message = result == null ? null : result.get("message");
    if (message instanceof Map<?,?> messageMap) {
      Object content = messageMap.get("content");
      return content == null ? "" : String.valueOf(content);
    }
    return "Ollama returned an empty response.";
  }
}
