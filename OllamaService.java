package local.doctorai;

import java.util.*;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.http.*;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestClient;

@Service
public class OllamaService {
  private final RestClient client;
  private final String model;
  public OllamaService(@Value("${ollama.base-url}") String baseUrl,
                       @Value("${ollama.model}") String model) {
    this.client = RestClient.builder().baseUrl(baseUrl).build();
    this.model = model;
  }
  public String model() { return model; }
  public Map<?,?> tags() {
    return client.get().uri("/api/tags").retrieve().body(Map.class);
  }
  public String chat(String prompt, List<String> images, String system) {
    Map<String,Object> user = new LinkedHashMap<>();
    user.put("role","user"); user.put("content",prompt == null ? "" : prompt);
    if (images != null && !images.isEmpty()) user.put("images", images);
    List<Map<String,Object>> messages = new ArrayList<>();
    messages.add(Map.of("role","system","content",system));
    messages.add(user);
    Map<String,Object> body = new LinkedHashMap<>();
    body.put("model",model); body.put("messages",messages); body.put("stream",false);
    body.put("keep_alive","10m");
    Map<?,?> result = client.post().uri("/api/chat")
      .contentType(MediaType.APPLICATION_JSON).body(body).retrieve().body(Map.class);
    Object message = result == null ? null : result.get("message");
    if (message instanceof Map<?,?> m) Object content = m.get("content"); return content == null ? "" : String.valueOf(content);
    return "Ollama returned an empty response.";
  }
}
