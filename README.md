# Doctor AI — Rust Desktop

Doctor AI is a native Rust desktop application for running supported AI models locally on Windows.

## Runtime architecture

- **Desktop application:** Rust + egui/eframe.
- **Inference runtime:** mistral.rs, built from Rust source and shipped with the installer.
- **Runtime communication:** Doctor AI starts and supervises the local inference process, probes its OpenAI-compatible API on a loopback-only ephemeral port, and streams responses into the native UI. No third-party model manager is required.
- **Model storage:** `%USERPROFILE%\\.doctor-ai\\models`.
- **Downloads:** streamed to a temporary `.part` file, resumed with HTTP Range when the source supports it, validated against the server's byte range/size, then atomically renamed.
- **Installer:** Inno Setup with a selectable destination folder. The installed runtime is native executables; no Java, Qt, Python, or PowerShell launcher is shipped or run on the user's PC.

## First run

1. Open the welcome page and review detected RAM, available memory, CPU, and free disk space.
2. Doctor AI recommends a model tier based on total and currently available memory.
3. The model page checks the app's model folder and known Hugging Face cache/download locations for the exact supported GGUF files.
4. Download a missing model from its source link; interrupted downloads retain partial data for resumption.
5. Start the local engine from the Chat page. The Send control is enabled only after the local API readiness probe succeeds.

## Included model catalogue

- **Gemma 4 E2B Q4_K_M:** lower-memory option for supported local chat.
- **Gemma 4 E4B Q4_K_M:** larger option for machines with more available RAM.

Model sizes and memory requirements vary by runtime, context length, and other applications. Confirm the model repository's current files, licence, and compatibility before downloading. The current release targets CPU inference on Windows; do not assume GPU acceleration.

## Local software discovery

The application checks known local model folders and common executable locations/PATH for selected tools, including ComfyUI, Blender, Krita, Whisper.cpp, Piper, and FFmpeg. Discovery is best-effort and does not claim a tool is installed when it cannot be found. The creative tools page also links to official project sites. These external programs and their model weights are optional and are not bundled in this installer.

## Build and test

Windows x64, Rust stable/MSVC, and Inno Setup 6:

```powershell
cargo build --manifest-path rust-app/Cargo.toml --release
cargo test --manifest-path rust-app/Cargo.toml
```

The GitHub Actions workflow builds the native app and inference runtime, runs Rust tests, embeds the branded executable icon, compiles the selectable Windows installer, and uploads the installer artifact.

## Verification boundary

A successful CI build proves that the code and installer compile and that included automated tests pass. It does not replace runtime validation on a physical Windows PC, tests with each downloaded model, performance benchmarks, or end-to-end tests for optional external tools. Those checks must be reported separately.
