# Doctor AI — local-first desktop application

Windows-first desktop app with a bundled Java backend and bundled llama.cpp CPU inference engine. Ollama is not required or used.

## Offline modules
- Available: Gemma 4 E2B / E4B local chat.
- Optional download: Gemma 4 image-understanding projector.
- Catalog only in this release: Whisper.cpp speech recognition, Piper text-to-speech, Stable Diffusion / ComfyUI image generation, Wan 2.x / ComfyUI video generation, and offline audio generation. These are not falsely presented as installed.

## Hardware guidance
- Desktop only: Windows 10/11 x64, 4 GB RAM, 3 GB free disk.
- Offline AI minimum: 8 GB RAM for Gemma 4 E2B.
- 8–15 GB RAM: Gemma 4 E2B Q4_0.
- 16 GB RAM or more: Gemma 4 E4B Q4_0.
- GPU is optional; GPU acceleration is not included in this CPU-engine release.

The installer checks RAM and disk before downloading. Bundled aria2 supports up to eight connections, retry, and resume.

## Developer build
Requires Qt 6.6+, CMake 3.24+, MSVC 2022, JDK 21 and Maven 3.9+.
```powershell
cmake -S . -B build -DCMAKE_PREFIX_PATH="$env:Qt6_DIR" -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
mvn -f backend/pom.xml test package
```

## Runtime
- llama-server listens only on 127.0.0.1:8080.
- Doctor AI backend listens only on 127.0.0.1:8787.
- Model files are stored in %USERPROFILE%\.doctor-ai\models.
- No Ollama, Python, Docker or separate Java install is required by the packaged app.

## Safety
Medical responses are decision support only and require clinician verification. This build performs local inference only.
