# Doctor AI — Windows Desktop starter project

A local-first desktop AI assistant using C++/Qt 6, Java 21/Spring Boot, Python media processing and Ollama.

## Repository layout
- Root: Qt 6 desktop app and CMake build.
- `backend/`: Java 21 Spring Boot API and SQLite chat history.
- `media/`: Python FastAPI/OpenCV service for extracting representative video frames.
- `skills/`: Markdown prompt skills loaded by the backend.

## Requirements
Windows 10/11 x64, Visual Studio 2022 C++ build tools, Qt 6.6+, CMake 3.24+, JDK 21, Maven 3.9+, Python 3.10+, FFmpeg and Ollama.

## Run Ollama
```powershell
ollama pull gemma4:12b
ollama serve
```
Ollama listens on `http://127.0.0.1:11434`.

## Run backend
```powershell
mvn -f backend/pom.xml spring-boot:run
```
Java listens on `http://127.0.0.1:8787`.

## Run optional media service
```powershell
cd media
py -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
uvicorn app:app --host 127.0.0.1 --port 8788
```
Video is handled by sampling up to 8 frames and sending them as images. This is not continuous native video/audio understanding.

## Build desktop
```powershell
cmake -S . -B build -DCMAKE_PREFIX_PATH="C:/Qt/6.8.3/msvc2022_64"
cmake --build build --config Release
```

## GitHub Actions
The workflow `.github/workflows/windows-build.yml` builds the Qt desktop app and Java backend on Windows and uploads a portable ZIP artifact. It does not yet produce a signed installer.

## Notes
- Services bind to loopback by default.
- Skills are prompt instructions, not model training.
- Do not expose local services to a network without authentication and TLS.
- Do not rely on an LLM as the sole basis for diagnosis or treatment.
