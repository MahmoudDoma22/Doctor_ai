# Doctor AI — Windows Desktop starter project

A local-first desktop AI assistant built with **C++ / Qt 6**, a **Java 21** backend,
**Python** media processing, and **Ollama** (`gemma4:12b`).

## Architecture

- `desktop/`: Qt 6 Widgets application. Welcome → Ollama connection → chat.
- `backend/`: Java 21 + Spring Boot REST API. Proxies chat requests to Ollama,
  discovers installed models, stores chat history in SQLite, and loads Markdown skills.
- `media/`: optional Python FastAPI service. Extracts representative frames from videos.
- `skills/`: user-editable Markdown skill files.

The desktop currently attaches image files and common video files (MP4/MOV/MKV/AVI; video payload limit 100 MB per file). The desktop talks only to the Java service (`http://127.0.0.1:8787`). Java talks to
Ollama (`http://127.0.0.1:11434`). Video frames are extracted by the Python service
(`http://127.0.0.1:8788`) and sent to Gemma as images.

## Requirements

- Windows 10/11 x64
- Qt 6.6+ with MSVC 2022 kit (or MinGW kit consistently used for all native builds)
- CMake 3.24+
- JDK 21
- Maven 3.9+
- Python 3.10+
- Ollama for Windows

## 1) Install and start the model

```powershell
ollama pull gemma4:12b
ollama serve
```

Ollama normally listens on `http://127.0.0.1:11434`.
Gemma 4 12B accepts text and images in Ollama. This project handles video by
sampling frames and sending them as images; it does not claim native continuous
video/audio understanding.

## 2) Start Java backend

```powershell
cd backend
mvn spring-boot:run
```

The backend listens on `http://127.0.0.1:8787`.
Override Ollama URL/model if needed:

```powershell
$env:OLLAMA_BASE_URL="http://127.0.0.1:11434"
$env:OLLAMA_MODEL="gemma4:12b"
mvn spring-boot:run
```

## 3) (Optional) Start Python video service

```powershell
cd media
py -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
uvicorn app:app --host 127.0.0.1 --port 8788
```

Requires FFmpeg installed and available on PATH. The service extracts up to 8
evenly-spaced frames. The backend can still handle text and images if this service
isn't running.

## 4) Build desktop application

Open a Developer PowerShell for VS 2022:

```powershell
cmake -S desktop -B desktop/build -DCMAKE_PREFIX_PATH="C:/Qt/6.8.*/msvc2022_64"
cmake --build desktop/build --config Release
```

Run `desktop/build/Release/DoctorAI.exe` (path may vary by generator).

For deployment, use Qt's `windeployqt` against the built executable, then package
the desktop app and backend runtime using the installer script in `packaging/`.
This source archive is **not a precompiled EXE**; a Windows build machine with
the requirements above is needed to produce the installer.

## API

- `GET /api/health` — checks backend and Ollama connectivity
- `GET /api/models` — lists installed Ollama models
- `POST /api/chat` — `{ "message": "...", "images": ["base64..."] }`
- `GET /api/skills` — lists loaded skills
- `POST /api/skills` — creates/updates a `.md` skill
- `GET /api/history` — recent saved turns

## Security and privacy notes

- Services bind to loopback by default; do not expose them to the LAN without
  authentication and TLS.
- Uploaded files are processed locally. Video frame files are temporary and removed.
- Skills are prompt instructions, not model training. They change behavior, not weights.
- Do not use an LLM as the sole basis for diagnosis or treatment decisions.
