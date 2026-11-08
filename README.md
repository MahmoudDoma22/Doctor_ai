# Doctor AI 2 — Rust-first desktop

Doctor AI is being rebuilt as a native Rust Windows desktop application.

## Architecture
- Native desktop UI: Rust + egui/eframe.
- Local inference runtime: mistral.rs (Rust).
- Local model storage: %USERPROFILE%\.doctor-ai\models.
- Model downloads: streamed HTTP with partial-file retention and HTTP Range resume when supported.
- No Java, Qt, Python, PowerShell installer scripts, or Ollama dependency in the new application path.

## User experience
- First-run hardware summary and model recommendation.
- Detects whether a selected model already exists.
- Model library with source links, download progress, and cancellation.
- Chat screen with streaming token display and a stop control beside Send.
- Notifications/status area and free image/video tool links.
- Inno Setup installer with a selectable installation directory and branded executable/installer icon.

## Model selection
- Gemma 4 E2B Q4_K_M: lighter local chat, 8 GB RAM class.
- Gemma 4 E4B Q4_K_M: higher memory requirement, 16 GB RAM class.
Model repository/file availability and exact memory needs must be verified on the target device. The model pages are the authoritative source for weights and licenses.

## Free creative tools
ComfyUI, Krita, Blender, Shotcut, Kdenlive, and Wan are linked from their official project pages. They are optional external applications, not bundled.

## Build
Windows x64, Rust stable/MSVC, and Inno Setup 6:
```powershell
cargo build --manifest-path rust-app/Cargo.toml --release
```

## Important
This is a rebuild foundation. GPU acceleration, robust model metadata discovery, complete conversation history, health/readiness checks, structured logs, and automated UI/inference tests remain gated on successful CI and runtime verification. Never treat an untested model download or engine startup as production-ready.
