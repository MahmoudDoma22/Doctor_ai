# Doctor AI — clean foundation

A Windows-first local AI desktop application. This branch is a clean rebuild, separate from the earlier prototype.

## Architecture
- Qt 6 / C++ desktop client
- Java 21 / Spring Boot local API
- Ollama local model runtime
- Future adapters for ComfyUI and local image/video/audio workflows

## Build prerequisites (developer machine)
- Windows 10/11 x64
- Qt 6.6+ with MSVC 2022
- CMake 3.24+
- JDK 21 and Maven 3.9+
- Ollama

## Build
```powershell
cmake -S . -B build -DCMAKE_PREFIX_PATH="$env:Qt6_DIR" -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
mvn -f backend/pom.xml test package
```

## Local services
Ollama listens on 127.0.0.1:11434. The backend listens only on 127.0.0.1:8787.
The app discovers installed Ollama models and selects one from the model dropdown.
Creative modules are not advertised as operational until a local generation engine is detected and tested.

## Safety
Medical responses are decision support only, not a substitute for clinician judgment. Patient-identifying data should not be entered unless the local privacy and retention configuration has been reviewed.
