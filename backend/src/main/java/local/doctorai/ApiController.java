package local.doctorai;
import java.util.*;
import org.springframework.http.*;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.client.RestClient;
@RestController
@RequestMapping("/api")
@CrossOrigin(origins="http://127.0.0.1")
public class ApiController {
 private final RestClient ollama;
 public ApiController(org.springframework.core.env.Environment env){ollama=RestClient.builder().baseUrl(env.getProperty("ollama.base-url","http://127.0.0.1:11434")).build();}
 @GetMapping("/health") public Map<String,Object> health(){try{Map<?,?> v=ollama.get().uri("/api/tags").retrieve().body(Map.class);return Map.of("ok",true,"ollama",true,"message","Ollama connected");}catch(Exception e){return Map.of("ok",false,"ollama",false,"message","Ollama is not reachable: "+e.getMessage());}}
 @GetMapping("/models") public Map<String,Object> models(){try{Map<?,?> v=ollama.get().uri("/api/tags").retrieve().body(Map.class);List<String> names=new ArrayList<>();Object raw=v==null?null:v.get("models");if(raw instanceof List<?> list)for(Object item:list)if(item instanceof Map<?,?> m&&m.get("name")!=null)names.add(m.get("name").toString());Collections.sort(names);String active=names.stream().filter(n->n.toLowerCase().contains("gemma")).findFirst().orElse(names.isEmpty()?"":names.get(0));return Map.of("models",names,"active",active);}catch(Exception e){return Map.of("models",List.of(),"active","");}}
 @PostMapping("/chat") public Map<String,Object> chat(@RequestBody Map<String,String> body){String prompt=body.getOrDefault("message","").trim();if(prompt.isEmpty())return Map.of("answer","Please enter a message.");String model=body.getOrDefault("model","");if(model.isBlank()){Map<String,Object> ms=models();model=String.valueOf(ms.get("active"));}if(model.isBlank())return Map.of("message","No Ollama model is installed.");try{Map<String,Object> payload=Map.of("model",model,"stream",false,"messages",List.of(Map.of("role","user","content",prompt)));Map<?,?> result=ollama.post().uri("/api/chat").contentType(MediaType.APPLICATION_JSON).body(payload).retrieve().body(Map.class);Object message=result==null?null:result.get("message");Object answer=message instanceof Map<?,?> m?m.get("content"):"";return Map.of("answer",String.valueOf(answer));}catch(Exception e){return Map.of("message","Local model request failed: "+e.getMessage());}}
}
