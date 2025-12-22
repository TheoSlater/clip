# Clipd

A cross-platform, low-latency **replay buffer daemon** built on FFmpeg.

Clipd continuously captures your screen (and audio), keeps a rolling in-memory buffer, and lets you instantly save the last _N seconds_ to disk — OBS/ShadowPlay style, but lightweight and hackable.

---

## ✨ Features

- Real-time screen capture (Windows, macOS, Linux)
- Rolling replay buffer (time-based, bounded memory)
- Instant clip saving (no re-encoding on clip)
- Hardware-accelerated encoding where available (NVENC on Windows)
- Designed for multi-monitor setups
- Clean HTTP API (perfect for hotkeys, tray apps, or Tauri UIs)
- Audio-ready architecture (system + mic, sync handled by FFmpeg)

---

## 🧠 How it works

1. FFmpeg captures screen (+ audio) in real time  
2. Encoded MPEG-TS stream is piped to stdout  
3. Stdout is ingested into an in-memory ring buffer  
4. Old packets are evicted based on time (e.g. last 30 seconds)  
5. `/clip` flushes the buffer directly to disk — instantly  

No decoding. No re-encoding. No waiting.

---

## 🏗 Architecture overview

- **FFmpeg supervisor**
  - Starts, restarts, and shuts down FFmpeg cleanly
  - Drains stdout and stderr to avoid deadlocks
- **Ring buffer**
  - Stores encoded packets with monotonic timestamps
  - Time-based eviction
- **HTTP API**
  - `/status` – daemon + buffer status
  - `/clip` – save the last _N seconds_
  - `/shutdown` – graceful shutdown
- **UI-agnostic**
  - Intended to be driven by a Tauri / native UI or global hotkeys

---

## 🖥 Platform support

### Windows
- Screen capture via `gdigrab`
- Hardware encoding via `h264_nvenc` (if available)
- System audio via WASAPI loopback (planned)
- Multi-monitor support (in progress)

### macOS
- Screen capture via `avfoundation`
- Requires Screen Recording permission

### Linux
- X11 capture via `x11grab`
- Wayland support will require PipeWire (future work)

---

## 🚀 Running locally

### Prerequisites

- Rust (stable)
- FFmpeg on PATH

### Start the daemon

    cargo run

The daemon listens on:

    http://127.0.0.1:43123

---

## 🔌 API

### `GET /status`

Returns current buffer and daemon state.

### `POST /clip`

Flushes the replay buffer to disk.

Response example:

    {
      "filename": "clip-2024-01-21_14-32-10.ts",
      "packets": 12456,
      "duration_ms": 29874,
      "bytes": 18239424
    }

The resulting `.ts` file is immediately playable.

### `POST /shutdown`

Gracefully stops FFmpeg and exits the daemon.

---

## 📦 Output format

- Clips are written as **MPEG-TS (`.ts`)**
- Remuxing to `.mkv` or `.mp4` can be done instantly without re-encoding

Example:

    ffmpeg -i clip.ts -c copy clip.mkv

---

## 🗺 Roadmap

- [ ] System audio capture (Windows, macOS, Linux)
- [ ] Microphone input with proper sync
- [ ] Monitor enumeration & selection
- [ ] Global hotkey support
- [ ] Tray / Tauri UI
- [ ] Configurable buffer length & encoding profiles
- [ ] Wayland support (PipeWire)

---

## ⚠ Notes

- This is a daemon, not a UI.
- Designed to be embedded into other apps.
- Cross-platform screen capture is inherently messy — FFmpeg is the least bad option.

---

## 📝 Licence

MIT (or whatever you decide — your project, your rules).

---

Built with Rust, FFmpeg, and stubbornness.
