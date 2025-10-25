# 🎬 SRTify – Offline AI Subtitle Generator

> Generate accurate subtitles for **video & audio files** — **100% offline**, powered by Whisper AI.

[![Download for Windows](https://img.shields.io/badge/Windows-0078D7?logo=windows&logoColor=white)](https://github.com/jebin2/SRTify/releases)
[![Download for macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white)](https://github.com/jebin2/SRTify/releases)
[![Download for Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black)](https://github.com/jebin2/SRTify/releases)

🌐 **Website**: [https://jebin2.github.io/SRTify/](https://jebin2.github.io/SRTify/)  
📥 **Latest Release**: [v0.1.0](https://github.com/jebin2/SRTify/releases/tag/app-v0.1.0)

---

## 🚀 Quick Start (End Users)

### 1. Download & Install

Go to **[Releases](https://github.com/jebin2/SRTify/releases/tag/app-v0.1.0)** and download the installer for your OS:
- **Windows**: `SRTify_v0.1.0_x64.msi` or `.exe`
- **macOS**: `SRTify_v0.1.0.dmg`
- **Linux**: `SRTify_v0.1.0.AppImage` or `.deb`

### 2. Windows Users: Fix Missing DLL (if needed)

If you see this error when launching:
> **`msvcp140.dll not found`**

👉 Install the **Microsoft Visual C++ Redistributable**:
🔗 [Download vc_redist.x64.exe](https://aka.ms/vs/17/release/vc_redist.x64.exe)

> ✅ This is a one-time system requirement for apps built with Visual Studio.

### 3. Run & Generate Subtitles
- Launch SRTify
- Select a **media file** (MP4, AVI, MP3, WAV, etc.)
- Choose an **output folder**
- Pick a **Whisper model** (e.g., `whisper-base`)
- Click **Generate Subtitle**

Outputs: `output.srt` and `output.json` with word-level timing.

---

## 🛠️ Build from Source (Developers)

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install)
- Build tools:
  - **Windows**: Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) or the [Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)
  - **Linux**: `sudo apt install build-essential cmake libssl-dev`
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)

### Setup Steps

```bash
git clone https://github.com/jebin2/SRTify.git
cd SRTify
npm install
```

### Prepare Dependencies

Create the dependency folder and populate it:

```bash
mkdir -p src-tauri/bin/dependency
```

Then copy binaries from your OS folder:
- **Linux**: `cp src-tauri/bin/linux-x64/* src-tauri/bin/dependency/`
- **Windows**: `cp src-tauri/bin/win32-x64/* src-tauri/bin/dependency/`
- **macOS**: `cp src-tauri/bin/macos-x64/* src-tauri/bin/dependency/`

> 💡 These must include:
> - `ffmpeg` (or `ffmpeg.exe`)
> - A Whisper model (e.g., `ggml-base.en.bin`)

> ⚠️ If the OS folders are missing, download:
> - [FFmpeg static builds](https://johnvansickle.com/ffmpeg/) (Linux/macOS) or [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) (Windows)
> - [Whisper models](https://huggingface.co/ggerganov/whisper.cpp/tree/main)

### Run in Dev Mode

```bash
npm run tauri dev
```

### Build Installer

```bash
npm run tauri build
```

Output installers will be in `src-tauri/target/release/bundle/`.

---

## 📦 Development Workflow

- **Auto-copy binaries**: The `beforeBuildCommand` runs `build_setup.js` to sync OS-specific binaries.
- **Push updates**: `npm run push`
- **Models**: Supports `tiny` → `large-v3-turbo` (downloaded on first use if not local)

---

## 🔐 Privacy Promise

✅ **All processing happens on your machine**  
✅ **No internet required after setup**  
✅ **No data leaves your device**

---

> Made with ❤️ using **Tauri + Rust + Whisper.cpp**