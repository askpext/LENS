# LENS

<img width="340" height="337" alt="Video Inspector Logo" src="https://github.com/user-attachments/assets/3d6415e3-317c-4bc2-9506-afb2231c32f0" />

> **Professional video analysis with a futuristic HUD interface.**  
> No spreadsheets. No clutter. Just pure technical insight.

<img width="1917" height="1076" alt="Video Inspector Dashboard" src="https://github.com/user-attachments/assets/57b16f83-0939-4e62-8359-b2bab65ea720" />
<img width="1251" height="868" alt="Screenshot 2025-12-24 152640" src="https://github.com/user-attachments/assets/959c7e0d-0ed0-4436-bb7a-797e2bac73cd" />

---

## ▣ CORE FEATURES

* **Visual Aspect Ratio** – Renders actual video proportions (16:9, 4:3, portrait)
* **Smart Gauges** – Bitrate and duration mapped to quality tiers (Web, HD, 4K)
* **Codec Badges** – Format and codec info as bold hardware identifiers
* **Instant Analysis** – Drag-and-drop powered by native **FFmpeg**
* **Lightweight** – Built on **Tauri** (Rust + WebKit). <15MB binary.
* **Cross-Platform** – Windows, macOS, Linux

---

## ⇩ DOWNLOAD

Get the latest build from **[Releases](../../releases)**

---

## ⚡ DEVELOPMENT

**Prerequisites:**
* [Rust](https://www.rust-lang.org/tools/install)
* [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

**Setup:**
```bash
# Clone
git clone https://github.com/yourusername/video-inspector.git
cd video-inspector

# Run Dev Mode
cargo tauri dev

# Build Production
cargo tauri build
```

**First Run:**  
FFmpeg auto-downloads on first launch (~50MB, one-time only)

---

## 📋 TECH STACK

* **Backend:** Rust, Tauri 2.0, ffmpeg-sidecar
* **Frontend:** Vanilla JS, CSS Glass Morphism
* **FFmpeg:** Embedded via automatic download


---

**Built with 🖤 by [Your Name]**
