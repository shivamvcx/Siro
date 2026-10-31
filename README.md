<div align="center">

  <br />

  <h3><b>SIRO</b></h3>

  <br />

  <img src="assets/icon_256.png" width="160" height="160" alt="Siro Icon" />

  <h1>Siro</h1>

  <p>
    <b>Hardware whiteboard and engineering planner: 100% offline, private, and built in pure Rust.</b><br />
    Schematic wiring, electronic components and pins, dynamic Bill of Materials, Notion-style tables,<br />
    and an infinite 2D canvas, in a native desktop app that opens instantly and works offline. Open source, and yours.
  </p>

  <p>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/100%25-Rust-DEA584?style=flat-square&logo=rust&logoColor=white" alt="100% Rust" /></a>
    <a href="#"><img src="https://img.shields.io/badge/Windows-native-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows Native" /></a>
    <a href="https://wgpu.rs/"><img src="https://img.shields.io/badge/graphics-wgpu-8A2BE2?style=flat-square" alt="wgpu Native" /></a>
    <a href="#"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue?style=flat-square" alt="License" /></a>
    <a href="#"><img src="https://img.shields.io/badge/status-beta-f59e0b?style=flat-square" alt="Status Beta" /></a>
    <a href="#"><img src="https://img.shields.io/badge/100%25-offline-2ea44f?style=flat-square" alt="100% Offline" /></a>
    <a href="#"><img src="https://img.shields.io/badge/telemetry-zero-0969da?style=flat-square" alt="Zero Telemetry" /></a>
  </p>

  <p>
    <a href="#-key-features"><b>Features</b></a> •
    <a href="#-native-siro-format"><b>File Format</b></a> •
    <a href="#-quick-start"><b>Quick Start</b></a> •
    <a href="#-building-from-source"><b>Building</b></a> •
    <a href="PROJECT_CONTEXT.md"><b>Master Context & Architecture</b></a>
  </p>

  <br />

</div>

<br />

## ✨ Key Features

| Capability | Description |
| :--- | :--- |
| **⚡ Instant Native Performance** | Built in pure Rust using `egui`/`eframe` and hardware-accelerated `wgpu`. Sub-second startup, buttery 60+ FPS interaction, and minimal memory footprint. |
| **🔌 Circuit & Electronics Library** | Pre-loaded with schematic components: Resistors, Capacitors, Inductors, Diodes, LEDs, Transistors, Generic ICs (custom pin counts), and Microcontrollers (ESP32, Pico, Arduino). |
| **📐 Orthogonal Schematic Wiring** | Intelligent 90° Manhattan wire routing with magnetic pin latching, visual terminal snapping, and soft anchor fallbacks. |
| **📋 Dynamic BOM & Hardware Kanban** | Integrated planning drawer: hardware Kanban boards (Backlog, Todo, In Progress, Done) and real-time Bill of Materials (BOM) counting directly from canvas components. |
| **📊 Notion-Style Data Tables** | In-canvas editable tables for register maps, pinout tables, and test matrices with per-cell text formatting and row/column operations. |
| **📝 Typography & Sticky Notes** | Figma-style text boxes that re-wrap without distorting font scale, plus colorful hardware lab sticky notes. |
| **🖼️ Image & Asset Embedding** | Import board schematics, oscilloscope traces, and mechanical drawings (PNG, JPEG, WebP) with perspective locking and corner handle resizing. |
| **🔒 100% Offline & Open Storage** | Transparent `.siro` container format (standard Deflate ZIP package) saving manifests, boards, and assets locally. Zero cloud lock-in. |
| **🛡️ Monotonic Close Guard** | Monotonic revision-counter dirty tracking and an unbreakable 3-option modal Close Guard intercepting window, tab, and Alt+F4 exit paths. |

<br />

## 📦 Native `.siro` Format

All projects are saved locally as open, inspectable `.siro` packages (standard PKZIP archive):

```text
my_hardware_project.siro (ZIP Archive)
├── manifest.json                     # Project UUID, schema version, ordered board IDs, timestamps
├── boards/                           # Canvas sheets directory
│   ├── e25c2793-....json             # Sheet 1: Elements, camera position, background style
│   └── 8a1f49b1-....json             # Sheet 2: Elements, camera position, background style
├── planner.json                      # Kanban goals/tasks + Bill of Materials items
└── assets/                           # Embedded binary assets (images, datasheets, PDFs)
    └── 3c49a1d2-..._schematic.png
```

- **Atomic Writes**: Writes to a hidden staging file `.<name>.tmp_<uuid>` before atomic rename, preventing file corruption.
- **Automatic Backups**: Creates `<name>.siro.bak` on every save.
- **Zero Base64 Bloat**: Binary images and datasheets are stored as raw compressed streams in `assets/`.

<br />

## 🚀 Quick Start

### Running Pre-Compiled Binaries
Siro ships as a standalone portable executable and standard installer:
- **Portable**: Run `siro_portable.exe` directly — zero installation or registry footprint.
- **Installer**: Run `Siro_Setup.exe` (or `Siro.msi`) for desktop shortcuts and `.siro` file associations.

### Developer Runner (`siro.bat`)
Launch the developer control center at the root of the workspace:
```cmd
siro.bat
```
- `[1]` Build and Launch (Debug)
- `[2]` Build and Launch (Release)
- `[3]` Fast Launch (No Rebuild — `< 1s`)
- `[4]` Full Verification (`cargo check` + `cargo test`)
- `[7]` Package Deliverables (Builds portable `.exe` + installers in `dist/`)

<br />

## 🛠️ Building from Source

### Prerequisites
- [Rust Stable Toolchain](https://rustup.rs/) (2021 edition)
- Windows 10/11 x64

### Compilation
```powershell
# Clone your repository
git clone https://github.com/your-username/siro.git
cd siro

# Run in debug mode
cargo run

# Run in optimized release mode (60+ FPS)
cargo run --release

# Run comprehensive automated test suite (119+ tests)
cargo test
```

<br />

## ⌨️ Global Shortcuts

| Shortcut | Action | Shortcut | Action |
| :--- | :--- | :--- | :--- |
| `V` | Select / Transform Tool | `Ctrl + S` | Save Project (`.siro`) |
| `P` | Freehand Pen Tool | `Ctrl + O` | Open File Dialog |
| `R` | Geometric Shapes Tool | `Ctrl + N` | New Project Tab |
| `T` | Text Box Tool | `Ctrl + W` | Close Tab (with Close Guard) |
| `W` | Orthogonal Wire Tool | `Ctrl + Z` | Undo |
| `E` | Precision Eraser Tool | `Ctrl + Y` | Redo (`Ctrl+Shift+Z`) |
| `Delete` | Delete Selected Elements | `Ctrl + D` | Duplicate Selection |
| `Space + Drag` | Pan Infinite Canvas | `Ctrl + +/-/0` | Zoom In / Out / 100% |

*Single-letter shortcuts are automatically suppressed while actively typing in text fields or table cells.*

<br />

## 📖 Deep Architecture & Full Context

For a comprehensive technical breakdown of all internal subsystems, invariants, and developer cookbooks:
👉 **[Read the Master Context Blueprint (`PROJECT_CONTEXT.md`)](PROJECT_CONTEXT.md)**

<br />

## 📄 License

Dual-licensed under either:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

---

<div align="center">
  <sub>Built with ❤️ for hardware engineers, makers, and STEM builders.</sub>
</div>
