# Siro — Master Project Context & Architectural Blueprint

> **Notice for Future AI Models & Developers**:  
> This single master context file contains the comprehensive architectural blueprint, design philosophy, file inventory, build instructions, core engineering invariants, deep-dive file format specifications, and a complete catalog of every feature in **Siro**. Reading this file alongside the source code in `src/` provides complete understanding of the entire application.

---

## Table of Contents
1. [Project Overview & Vision](#1-project-overview--vision)
2. [Technology Stack & Dependencies](#2-technology-stack--dependencies)
3. [Build, Run, Test, and Distribution Instructions](#3-build-run-test-and-distribution-instructions)
4. [Codebase Architecture & Directory Structure](#4-codebase-architecture--directory-structure)
5. [Comprehensive Feature Catalog & Behavioral Rules](#5-comprehensive-feature-catalog--behavioral-rules)
   - [5.1 Canvas & Creative Tools](#51-canvas--creative-tools)
   - [5.2 Project, Multi-Sheet & Planning Architecture](#52-project-multi-sheet--planning-architecture)
   - [5.3 Data Storage, Persistence & Safety](#53-data-storage-persistence--safety)
   - [5.4 Home Screen Dashboard & File Management](#54-home-screen-dashboard--file-management)
   - [5.5 User Interface, Theming & Controls](#55-user-interface-theming--controls)
6. [Deep-Dive: Native `.siro` File Format & Storage Engine](#6-deep-dive-native-siro-file-format--storage-engine)
7. [Application Execution Lifecycle & Frame Loop](#7-application-execution-lifecycle--frame-loop)
8. [Key Subsystems & Architectural Invariants](#8-key-subsystems--architectural-invariants)
9. [Developer Extensibility Cookbooks](#9-developer-extensibility-cookbooks)
10. [Local Storage & Environment Paths](#10-local-storage--environment-paths)
11. [Testing & Quality Assurance](#11-testing--quality-assurance)
12. [Guidelines for AI Agents & Future Contributors](#12-guidelines-for-ai-agents--future-contributors)

---

## 1. Project Overview & Vision

**Siro** is a high-performance, single-user, 100% offline desktop whiteboard and engineering planner tailored specifically for STEM practitioners, embedded systems developers, and hardware builders.

### Core Tenets
1. **100% Offline & Private**: Zero cloud dependencies, zero telemetry, zero analytics, zero external network calls. All data is saved locally on disk in open, inspectable format.
2. **Instant & Native Performance**: Built in Rust using `egui` / `eframe` with hardware-accelerated `wgpu` rendering. Achieves sub-second startup and consistent 60+ FPS interaction on infinite 2D canvas boards.
3. **Hardware & STEM First**: Dedicated tools for drawing orthogonal circuit wires, placing electronic components (resistors, ICs, microcontrollers, diodes, capacitors) with interactive connection pins, building Bill of Materials (BOM), and managing hardware task boards.
4. **UI Honesty**: No fake cloud UI, no mock workspace selectors, no dummy team collaboration avatars, and no dead buttons. Every visual control has real backend functionality.
5. **Data Safety**: Protected by monotonic revision-counter dirty tracking, configurable atomic autosave sweeps, backup file retention (`.siro.bak`), and a modal Close Guard that intercepts window/tab closing when unsaved changes exist.

---

## 2. Technology Stack & Dependencies

- **Language & Edition**: Rust 2021 Edition (`Cargo.toml`).
- **GUI & Graphics**:
  - `eframe 0.31` & `egui 0.31`: Immediate-mode GUI framework configured with `wgpu` rendering backend, `default_fonts`, and `persistence`.
- **Serialization & Packaging**:
  - `serde 1.0` & `serde_json 1.0`: Model serialization to JSON.
  - `zip 2.6` (deflate): Native `.siro` container file packaging.
  - `uuid 1.10` (`v4`, `serde`): Unique entity and project identifiers.
  - `chrono 0.4` (`serde`): Timestamps for manifests, modified dates, and task tracking.
- **Native OS Integration**:
  - `rfd 0.15`: Native file open and save dialogs.
  - `image 0.25`: PNG, JPEG, WebP decoding for canvas image elements.
  - `windows 0.58`: Windows platform integration (`Data_Pdf`, `Storage_Streams`, `Storage`).
  - `winres 0.1`: Windows binary resource compilation (`siro.rc`, `app_icon.ico`, `app.manifest`).
- **Packaging & Distribution Tools**:
  - `Inno Setup 6`: Generates standard Windows installer (`installer/siro_setup.iss`).
  - `WiX Toolset v3`: Generates enterprise MSI package (`installer/siro.wxs`).
  - `PowerShell 5.1+`: Packaging orchestration script (`scripts/package.ps1`).
  - `siro.bat`: Developer control center batch script for building, testing, cleaning, and running.

---

## 3. Build, Run, Test, and Distribution Instructions

### Prerequisites
- Rust stable toolchain (`rustup`, `cargo`, `rustc`).
- Windows 10/11 x64 (primary supported OS).

### Quick Commands
```powershell
# Run in debug mode (development)
cargo run

# Run in optimized release mode (60 FPS production build)
cargo run --release

# Run compile check across all targets and tests
cargo check --all-targets

# Run the comprehensive automated test suite (119+ tests)
cargo test
```

### Developer CLI Tool (`siro.bat`)
A helper script `siro.bat` is provided at the repository root:
- `siro.bat 1` — **Build and Launch (Debug)**: Automatically kills any running instance, builds, and launches.
- `siro.bat 2` — **Build and Launch (Release)**: Builds optimized release binary and launches.
- `siro.bat 3` — **Fast Launch (No Rebuild)**: Instantly starts the existing compiled binary in `< 1s`.
- `siro.bat 4` — **Full Verification**: Executes `cargo check --all-targets` and `cargo test`.
- `siro.bat 5` — **Kill Running App**: Terminates lingering `siro.exe` background processes.
- `siro.bat 6` — **Clean Build Cache**: Safely clears `target/` without affecting source or config.
- `siro.bat 7` — **Package Deliverables**: Invokes `scripts/package.ps1` to produce portable binary and installers in `dist/`.

---

## 4. Codebase Architecture & Directory Structure

```text
Siro Antigravity/
├── Cargo.toml                  # Cargo manifest with dependencies and release profile
├── Cargo.lock                  # Dependency lockfile
├── build.rs                    # Windows resource compiler linking manifest and icons
├── siro.bat                    # Developer CLI batch script
├── icon.svg                    # Vector brand mark for Siro
├── assets/                     # High-res icon assets (icon.ico, icon_256.png, icon_32.png)
├── installer/                  # Packaging metadata (WiX, Inno Setup, manifest, rc file)
├── scripts/                    # Build scripts (build_icons.js, package.ps1)
├── src/                        # Main Rust source tree
│   ├── main.rs                 # Native application entry point & window configuration
│   ├── lib.rs                  # Library root exposing modules & APP_NAME constant
│   ├── app.rs                  # SiroApp state machine, event loop, tabs, close guard
│   ├── shortcuts.rs            # Keyboard shortcut registry and key conflict resolution
│   ├── bin/
│   │   └── gen_icons.rs        # Standalone binary for rasterizing icons from vector data
│   ├── canvas/                 # 2D infinite canvas subsystem
│   │   ├── camera.rs           # 2D viewport pan/zoom transforms & coordinate math
│   │   ├── grid.rs             # Infinite dynamic dot/lined grid rendering
│   │   ├── render.rs           # Canvas painting pipeline using egui::Painter
│   │   ├── snap.rs             # Smart object-to-object & grid snapping engine
│   │   └── tools/              # Interactive canvas tool handlers
│   │       ├── select.rs       # Marquee selection, 8-handle scaling, rotation, translation
│   │       ├── pen.rs          # Freehand drawing with stroke smoothing
│   │       ├── shapes.rs       # Rectangles, rounded rects, ellipses, diamonds
│   │       ├── text_tool.rs    # Floating text box placement and editing
│   │       ├── table_tool.rs   # Grid data tables with configurable rows & columns
│   │       ├── wire.rs         # Orthogonal schematic wiring between component pins
│   │       └── eraser.rs       # Hit-testing eraser for elements and strokes
│   ├── model/                  # Core domain models
│   │   ├── project.rs          # Project container, revision tracking, display names
│   │   ├── board.rs            # Board container, element collections, background style
│   │   ├── element.rs          # CanvasElement enum, geometries, styles, bounds
│   │   ├── electronics.rs      # Electronics component definitions, pins, and symbols
│   │   ├── planner.rs          # Kanban task boards, goals, Bill of Materials (BOM)
│   │   ├── sheet_ops.rs        # Multi-sheet operations (add, duplicate, delete, reorder)
│   │   └── undo.rs             # Monotonic undo/redo snapshot stack
│   ├── storage/                # File persistence, export, and settings
│   │   ├── siro_file.rs        # Native .siro ZIP container read/write & backup (.bak)
│   │   ├── settings.rs         # AppSettings persistence (%LOCALAPPDATA%/Siro/settings.json)
│   │   ├── autosave.rs         # Background autosave interval sweep engine
│   │   ├── history.rs          # Recent files & starred boards tracking
│   │   ├── export.rs           # High-resolution export to PNG, SVG, and PDF
│   │   └── pdf_worker.rs       # Background worker thread for asynchronous PDF generation
│   └── ui/                     # UI components and layout
│       ├── home_screen.rs      # Dashboard: recents, starred, template picker, settings
│       ├── top_bar.rs          # Top navigation bar, tabs, title rename, status pill
│       ├── tool_rail.rs        # Left vertical tool rail & component drawer
│       ├── properties_bar.rs   # Floating contextual properties bar for selection
│       ├── planner_panel.rs    # Collapsible side drawer for Kanban & BOM
│       ├── close_dialog.rs     # Modal Close Guard dialog (Save, Discard, Cancel)
│       ├── color_presets.rs    # Curated color palettes for STEM & hardware design
│       └── popup_manager.rs    # Contextual overlays, menus, and dropdowns
└── tests/                      # Automated test suite
    ├── autosave_and_close_tests.rs # Revision counter, close plan, and settings tests
    ├── canvas_tests.rs             # Interaction, UI honesty, theme persistence tests
    ├── new_features_tests.rs       # Table, wiring, electronics, and multi-sheet tests
    ├── snap_tests.rs               # Snapping alignment and grid calculation tests
    └── undo_and_model_tests.rs     # Model mutations and undo/redo invariant tests
```

---

## 5. Comprehensive Feature Catalog & Behavioral Rules

This catalog details every individual feature in Siro across all application subsystems. Each feature entry explains its purpose, interaction semantics, and inviolable engineering rules.

### 5.1. Canvas & Creative Tools

#### 1. Selection & Transformation Engine (`Select` Tool)
The Selection tool provides marquee bounding-box selection, element dragging, multi-element aggregation, and 8-point boundary scaling. Clicking an unselected element selects it; clicking empty canvas deselects; dragging across empty space draws a marquee rectangle selecting all intersecting elements. When elements are selected, an 8-handle bounding box appears supporting directional scaling and corner resizing.
- **Expected Behavior**: Hovering handles displays directional cursor icons (`nwse`, `nesw`, `ew`, `ns`). Dragging a handle dynamically stretches the element bounds while maintaining anchor position at the opposite handle. Holding `Shift` preserves the original aspect ratio. Dragging inside the selection translates all selected elements synchronously with subpixel precision. Pressing `Delete` deletes selected items without deleting any active text editing buffer.
- **Rules & Invariants**: Must record undo snapshots before dragging starts (`snapshot_initial_state`). Must call `project.mark_edited()` on transform completion. Aspect-ratio locked elements (e.g. images with lock enabled) must honor aspect constraints even if `Shift` is unpressed.

#### 2. Smooth Freehand Pen Tool (`Pen` Tool)
The Pen tool provides natural freehand vector sketching on the canvas for diagrams, handwritten annotations, and sketches. As the user drags the mouse or stylus, raw screen points are converted to world coordinates and collected in real time.
- **Expected Behavior**: Live drawing shows continuous stroke preview matching selected stroke width and color. On mouse release, the raw point stream is automatically simplified using the Ramer-Douglas-Peucker (RDP) algorithm with a threshold (`epsilon ≈ 1.5px`) to reduce point density while preserving curves, then committed as a `CanvasElement::Stroke`.
- **Rules & Invariants**: Empty or single-point strokes on click without drag must be discarded or registered as minimal dots. Simplification must never distort sharp corners beyond the configured epsilon. Stroke generation must invoke `project.mark_edited()`.

#### 3. Geometric Shapes Engine (`Shapes` Tool)
The Shapes tool enables placing foundational vector geometric elements: Rectangles, Rounded Rectangles, Ellipses/Circles, and Diamonds (Flowchart decision symbols). The user selects a shape type from the tool rail or flyout, then clicks and drags on the canvas to define position and dimensions.
- **Expected Behavior**: Live rubber-band preview renders with current stroke and fill settings as the cursor moves. Releasing mouse commits the element. The shape can subsequently be moved, resized via bounding handles, or recolored via the contextual properties bar. Rounded rectangles support adjustable corner radii.
- **Rules & Invariants**: Minimum bounding dimensions are clamped (width >= 4px, height >= 4px) to prevent zero-area or negative-dimension inversions. Geometry normalization ensures `rect.min <= rect.max` regardless of drag direction (top-left vs bottom-right).

#### 4. Figma-Style Text Box & Inline Typography (`Text` Tool)
The Text tool allows adding floating text blocks, section headers, and notes anywhere on the infinite canvas. Clicking on the canvas creates a text box in active editing mode, autofocusing the keyboard input.
- **Expected Behavior**: Text editing supports multi-line text, standard cursor positioning, selections, copy/paste, and font size choices. Resizing the text box via side handles re-wraps the text flow (Figma style) without stretching or distorting the font glyph scale. The text box supports customizable background fill colors with padding.
- **Rules & Invariants**: When the user is actively typing in a text box, pressing the `Delete` or `Backspace` keys must delete text characters only and must NEVER trigger canvas element deletion. Deselecting the text box commits the text and trims empty whitespace-only elements.

#### 5. Hardware Sticky Notes
Sticky Notes provide colored square and rectangular cards designed for quick brainstorming, pinout reminders, and lab checklist items. Available in curated hardware and paper color presets.
- **Expected Behavior**: Clicking with the Sticky Note tool spawns a pre-sized note (default 160x160px) with centered, auto-wrapping text. Clicking directly on a note initiates inline text editing. Notes can be stacked, grouped, and repositioned freely.
- **Rules & Invariants**: Text must wrap strictly within the note boundaries with standard internal padding (8px). Notes maintain a minimum size to prevent illegible compression.

#### 6. Notion-Style Data Tables (`Table` Tool)
The Table tool renders spreadsheet-like tabular data grids directly on the canvas, ideal for pinout tables, register maps, test matrices, and Bill of Materials (BOM). Clicking on the canvas places a customizable $M \times N$ table.
- **Expected Behavior**: Hovering columns or rows reveals insertion buttons (`+`) and option menus. Cells support direct text editing with `Tab` navigating between cells. Columns support drag-to-resize borders. Context menus allow inserting/deleting rows and columns, recoloring cell backgrounds, and clearing contents. Menu popups stay open reliably when moving the cursor from row headers into the options popup.
- **Rules & Invariants**: Tables must always maintain at least 1 row and 1 column. Total table width must match the sum of its column widths. Deleting a row or column must invoke `project.mark_edited()` and register an undo step.

#### 7. Image Import & Asset Management
The Image tool enables importing external raster graphics (PNG, JPEG, WebP) directly onto the canvas, including board schematics, component pinouts, oscilloscope screenshots, and mechanical drawings.
- **Expected Behavior**: Triggered via file picker or drag-and-drop. Images are decoded, dimensions measured, and scaled to reasonable default canvas bounds. Users can drag 8-point handles to resize. A toggle button in the floating properties bar allows locking the aspect ratio or resetting perspective distortion to original dimensions.
- **Rules & Invariants**: Binary image bytes must be safely stored in the project's internal `assets/` directory inside the `.siro` ZIP package, identified by UUID. Canvas elements reference assets by UUID to avoid bloating JSON board definitions with Base64 payloads.

#### 8. Hardware & STEM Electronics Component Library
A built-in electronic component catalog providing schematic symbols for hardware developers: Resistors, Capacitors, Inductors, Diodes, LEDs, Transistors (NPN, PNP, MOSFET), Generic Integrated Circuits (ICs), and Microcontroller dev boards (ESP32, Raspberry Pi Pico, Arduino).
- **Expected Behavior**: Selecting a component from the tool rail places it on canvas with standard schematic symbols, terminal connection pins, reference designators (e.g. `R1`, `C1`, `U1`), and component values. Components support 90-degree rotational steps (`R` key or rotate button). Generic ICs allow custom pin counts (e.g. DIP-8, DIP-16, QFP) and custom pin naming.
- **Rules & Invariants**: Pin counts for `GenericIC` must be clamped to $\ge 2$ and must be even numbers to ensure symmetrical dual-in-line layout and prevent division by zero in step calculations. Terminal pin world positions must transform accurately with component translation and rotation.

#### 9. Orthogonal Schematic Circuit Wiring Tool (`Wire` Tool)
The Circuit Wiring tool provides schematic wiring between electronic component terminals or freeform canvas points.
- **Expected Behavior**: Moving near a component pin highlights a magnetic terminal latch target. Clicking begins routing. As the user moves the cursor, the tool calculates orthogonal (Manhattan / 90-degree) paths with clean corners. Clicking pins completes the connection, establishing an associative link between components. Clicking empty canvas adds an intermediate waypoint.
- **Rules & Invariants**: Magnetic wire latching must use a snap radius threshold (default 10-14px). When no terminal is in range, wire endpoints must fall back smoothly to soft anchor points on element bounding perimeters. Segment hit-testing must accurately detect clicks within 5px of any line segment for selection or erasure.

#### 10. Precision Eraser Tool (`Eraser` Tool)
The Eraser tool removes elements and strokes from the canvas. Provides two distinct modes: Whole Object Eraser and Freehand Stroke Eraser.
- **Expected Behavior**: In Object Eraser mode, clicking or sweeping over any element deletes it entirely. In Stroke Eraser mode, sweeping over pen strokes splits or erases contacted segments with spatial hit-testing. Hovering over elements during erase mode provides visual red/highlight feedback indicating what will be deleted.
- **Rules & Invariants**: Erasing elements must register a single atomic undo action representing all elements deleted in that swipe. Eraser operations must call `project.mark_edited()`.

#### 11. Infinite 2D Canvas & Anchor-Preserving Camera
The canvas subsystem provides a virtually boundless 2D workspace with coordinate transformations between window screen coordinates and world space.
- **Expected Behavior**: Panning is supported via Middle Mouse drag, `Space + Left Mouse drag`, or two-finger trackpad drag. Zooming is supported via Mouse Wheel, `Ctrl + +`, `Ctrl + -`, or the zoom controls in the top bar (from 10% up to 500%). The camera uses anchor-preserving zoom, keeping the world point under the cursor stationary on screen during zoom operations.
- **Rules & Invariants**: `screen_to_world` and `world_to_screen` transforms must maintain subpixel floating-point precision roundtrips without floating-point drift. Viewport frustum culling must skip rendering elements completely outside the visible screen rectangle to maintain 60 FPS performance on boards with thousands of elements.

#### 12. Adaptive Dynamic Grid Subsystem
Renders an infinite background grid that adapts dynamically to the camera zoom level.
- **Expected Behavior**: Supports two visual modes: Dot Grid and Lined Grid, plus an option to toggle grid visibility Off. As the camera zooms out, fine grid lines fade out smoothly to avoid visual moiré patterns and clutter, while primary grid lines remain visible. As the camera zooms in, fine sub-grid divisions emerge seamlessly.
- **Rules & Invariants**: Grid spacing math must align with canvas snap targets. Grid rendering must execute with minimal draw calls and zero CPU allocations per frame.

#### 13. Smart Snapping & Dynamic Alignment Guidelines
An intelligent geometry snapping engine that assists in aligning elements to grid lines and to neighboring elements during dragging and resizing.
- **Expected Behavior**: When dragging or resizing an element, its edges (left, right, top, bottom) and center axes (X, Y) are compared against other elements on the board. When within snapping threshold, the element snaps into alignment and bright cyan guideline overlays appear showing the exact alignment match. Snapping also checks canvas grid lines.
- **Rules & Invariants**: Snapping must be zoom-invariant: the snap threshold is computed in screen pixels (e.g. 8px) so snapping feels consistent regardless of whether the user is zoomed in to 200% or out to 25%. Independent axes: snapping on X must not lock or constrain free movement on Y unless Y also hits a candidate. Snapping can be temporarily suppressed by holding `Alt`.

---

### 5.2. Project, Multi-Sheet & Planning Architecture

#### 14. Multi-Project Tab Strip
Enables opening and managing multiple `.siro` project files simultaneously within a single window, displayed as tabs across the top navigation bar.
- **Expected Behavior**: Clicking tabs switches the active project instantly. Clicking `+` creates a new untitled project tab. Each tab displays its project display name and an unsaved modification dot (`" •"`) when edits exist. Hovering reveals a close button (`✕`). Switching tabs when the active project has unsaved changes and an existing file path automatically saves it to disk before switching.
- **Rules & Invariants**: Each open project must possess a distinct UUID. Closing a dirty tab must route through the Close Guard modal dialog. Closing the last remaining project tab transitions the application cleanly back to the Home Dashboard (`AppState::Home`).

#### 15. Multi-Sheet / Multi-Board Subsystem
Allows a single `.siro` project to contain multiple independent canvas sheets (e.g. "Main Schematic", "Power Stage", "Mechanical Notes").
- **Expected Behavior**: The bottom/top bar provides a sheet switcher dropdown. Users can switch sheets, add new sheets (`+`), duplicate existing sheets with all elements, rename sheets, and delete sheets. Renaming enforces validation with immediate inline feedback.
- **Rules & Invariants**: Sheet names must be non-empty, trimmed of whitespace/control characters, limited to 64 characters, and case-insensitively unique within the project. A project must always retain at least one sheet (the last sheet cannot be deleted). Duplicating a sheet regenerates unique UUIDs for all duplicated elements while preserving relative spatial layout.

#### 16. Integrated Kanban Project Planner & Bill of Materials (BOM)
A collapsible right-hand drawer that combines hardware project task tracking with an automated Bill of Materials calculator.
- **Expected Behavior**: The Planner view provides Kanban columns (Backlog, Todo, In Progress, Done) with cards supporting titles, descriptions, priority tags, and due dates. The BOM view automatically aggregates all electronic components placed on the canvas sheets, counting quantities, footprint packages, and values (e.g. "10kΩ 0805: 4 pcs"). Users can also add custom non-canvas hardware items (screws, enclosures, connectors).
- **Rules & Invariants**: Mutations to Kanban tasks or BOM rows call `project.mark_edited()`. BOM generation must deduplicate components based on matching component type, value, and footprint. Default empty planner projects must evaluate `has_content == false` so pristine files do not prompt unsaved warnings.

#### 17. Transactional Undo / Redo History Stack
A robust command-pattern undo/redo subsystem that tracks all canvas modifications.
- **Expected Behavior**: Pressing `Ctrl+Z` reverses the last action; pressing `Ctrl+Y` or `Ctrl+Shift+Z` re-applies it. Covers element additions, deletions, translations, resize transformations, property recoloring, and multi-element operations. The top bar provides dedicated clickable Undo and Redo pill buttons with live enabled/disabled visual states.
- **Rules & Invariants**: Undo stack has a bounded capacity (default 50 steps) to prevent memory exhaustion. Typing in an active text box must not flood the undo stack with single keystroke snapshots; text edits commit as coherent units on blur or exit.

---

### 5.3. Data Storage, Persistence & Safety

#### 18. Native `.siro` ZIP Archive Format & Atomic Persistence
The native open file container format for all Siro projects.
- **Expected Behavior**: A `.siro` file is a standard deflate-compressed ZIP archive containing `manifest.json` (metadata, board list), `board_<id>.json` (elements per sheet), `planner.json` (Kanban tasks and BOM items), and an `assets/` directory (embedded images).
- **Rules & Invariants**: Saves must write to a temporary file (`.siro.tmp_<uuid>`) and atomically replace the destination file upon successful completion, ensuring files are never corrupted by mid-write crashes or power loss. File paths are strictly enforced to have the `.siro` extension.

#### 19. Automated Safety Backups (`.siro.bak`)
An automatic disk-level backup mechanism safeguarding user data on every save operation.
- **Expected Behavior**: Whenever a project file `project.siro` is saved to disk, any existing copy on disk is copied to `project.siro.bak` prior to overwriting.
- **Rules & Invariants**: If a project file is permanently deleted via the in-app delete modal, the corresponding `.siro.bak` file is also pruned from disk. When renaming a project on disk, the `.siro.bak` file is renamed in tandem.

#### 20. Monotonic Revision-Counter Dirty State Engine
Replaces legacy, fragile boolean `dirty` flags with a strict monotonic revision-counter system.
- **Expected Behavior**: The UI tab dirty dot (`" •"`) and floating capsule indicator (`"Local • Saved"` vs `"Local • Edited"`) derive exclusively from `project.is_unsaved()`.
- **Rules & Invariants**: `Project` maintains `edit_revision: u64` and `saved_revision: u64`. `mark_edited()` increments `edit_revision`. `mark_saved()` sets `saved_revision = edit_revision` strictly after archive disk write succeeds. Pure function `is_unsaved(&Project) -> bool` returns `edit_revision != saved_revision`. `has_content(&Project) -> bool` returns true only if the project contains actual elements, renamed sheets, or non-default planner data.

#### 21. 3-Action Modal Close Guard & Window Interception
Guarantees user data is never lost by intercepting all application, window, and tab exit requests.
- **Expected Behavior**: Intercepts the OS window close button (✕), Taskbar close, `Alt+F4`, tab close buttons, and file menu close actions. If unsaved changes exist in any project with content, a modal dialog appears offering 3 explicit buttons:
  1. **Save then exit / close**: Saves all unsaved files (triggering file pickers for untitled boards) and proceeds with close.
  2. **Discard changes**: Ignores changes and exits cleanly.
  3. **Cancel**: Closes dialog, stays in the application.
- **Rules & Invariants**: Window close is intercepted via `ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose)`. Only when clean or after user confirmation is `self.allow_close = true` set and `Close` emitted. Pressing `Esc` maps strictly to **Cancel**.

#### 22. Non-Blocking Background Autosave Engine
Periodic background persistence engine that protects active work without interrupting user interactions.
- **Expected Behavior**: Evaluates open projects at the end of each frame. For any project where `is_autosave_eligible(&project)` is true (meaning it has an established disk `filepath` and `is_unsaved() == true`), it persists changes silently to disk according to the configured interval.
- **Rules & Invariants**: Does not trigger file picker dialogs for untitled projects without an existing file path. Does not block the main UI render loop. Configurable interval between 5 seconds and 600 seconds, or can be turned Off.

#### 23. Persistent User Application Settings (`settings.json`)
Persistent application-level configuration management.
- **Expected Behavior**: Settings are stored in `%LOCALAPPDATA%\Siro\settings.json` (or `~/.siro/settings.json`). Configurable settings include autosave interval (`Option<u64>` where `None` = Off, `Some(5..=600)` = seconds).
- **Rules & Invariants**: Settings are written atomically via temporary file and rename. Tolerant loading fallback: missing, unreadable, or invalid JSON falls back gracefully to default (`60` seconds) without crashing or alerting errors.

#### 24. Multi-Format Canvas Export (PNG, SVG, Asynchronous PDF)
High-resolution vector and raster export pipeline for documentation, schematics, and manufacturing reviews.
- **Expected Behavior**: Users can export the active board or selection via the top bar export menu to:
  1. **PNG Image Snapshot**: 300 DPI raster snapshot with transparent or solid background.
  2. **Vector SVG**: Clean, infinitely scalable vector curves and text.
  3. **PDF Document**: Formatted PDF document generated asynchronously via a background worker thread.
- **Rules & Invariants**: PDF generation runs on an independent worker thread communicating via message channels to prevent the UI from freezing on large boards. Default export filename derives from `project.display_name()`.

---

### 5.4. Home Screen Dashboard & File Management

#### 25. Central Home Dashboard & Sidebar Navigation
The landing dashboard displayed on application launch or when clicking the Home logo.
- **Expected Behavior**: Features a clean sidebar providing one-click routing between:
  - **Home**: Main dashboard with quick templates and recent boards.
  - **Starred**: Filtered view showing only favorite starred boards.
  - **Browse**: In-app local filesystem explorer.
  - **Settings (⚙)**: Pinned to bottom-left, opens autosave interval configuration.
- **Rules & Invariants**: Navigating to Home does not close open project tabs; tabs remain accessible in the top strip. The Home screen adapts responsively to narrow viewports without clipping navigation items.

#### 26. Instant Global Project Search
A real-time search bar located prominently at the top of the Home dashboard.
- **Expected Behavior**: Typing in the search input instantly filters project cards and table rows in real time across title names and file paths. A clear button (`✕`) appears when text is present to reset the filter instantly.
- **Rules & Invariants**: Search filtering is case-insensitive. Empty search displays the complete file list. Shows a helpful empty-state message when no matching projects are found.

#### 27. Hardware Engineering Starter Templates Gallery
A gallery of pre-built, production-quality project templates designed for rapid STEM project scaffolding.
- **Expected Behavior**: The Home screen provides 5 pre-configured template cards:
  1. **IoT Sensor Node**: Microcontroller, power regulation, sensor buses (I2C/SPI), BOM, and firmware tasks.
  2. **Benchtop Power Supply**: Transformer, rectifier, linear/switching regulators, heatsink calculations.
  3. **Robotics Motor Controller**: H-bridge MOSFETs, gate drivers, PWM inputs, current sense resistors.
  4. **Audio Preamplifier**: Low-noise op-amps, input filters, volume potentiometer, symmetric power rails.
  5. **Embedded Systems Test Fixture**: Pogo pins, test points, breakout headers, validation checklist.
- **Rules & Invariants**: Clicking a template instantiates a new untitled project tab pre-populated with components, schematic wires, notes, and Kanban tasks, immediately ready for customization.

#### 28. Recent Files Explorer with Honest Metadata
A comprehensive view of recently opened `.siro` projects on the user's machine.
- **Expected Behavior**: Provides a toggle between Card Grid View and Detailed Table View. Displays genuine filesystem metadata for every file: exact file size in KB/MB and actual disk last-modified date and time. Clicking a card or row opens the project.
- **Rules & Invariants**: Strictly displays honest filesystem metadata derived from `std::fs::metadata` (no hardcoded dates or fake dummy space names). Missing or deleted files show a "Not Found" indicator with an option to prune from history. Uses an LRU list capped at 50 entries in `%LOCALAPPDATA%\Siro\recent_files.json`.

#### 29. Starred Projects Favoriting System
A favorites system allowing users to pin essential or active project boards for rapid access.
- **Expected Behavior**: Project cards and table rows feature an interactive 5-point vector star icon. Clicking toggles favorite status. Starred projects render with a gold fill (`#f59e0b`); unstarred projects render with a subtle outline.
- **Rules & Invariants**: Vector star is rendered with exact 7.0px geometric radius in both starred and unstarred states. Starred file paths are stored persistently in `%LOCALAPPDATA%\Siro\starred_files.json`.

#### 30. Local Filesystem Project Browser
An integrated directory browser enabling exploration of local folders to discover `.siro` boards.
- **Expected Behavior**: Clicking "Browse" opens a folder navigation tree. Users can navigate up and down directory structures, view `.siro` files, and open them directly with one click.
- **Rules & Invariants**: Directory sorting lists subdirectories first (alphabetically), followed by `.siro` files. Handles long paths, deep hierarchies, and non-ASCII UTF-8 folder names safely.

#### 31. Permanent File Deletion with Confirmation Modal
A secure file deletion mechanism that permanently deletes project files from disk.
- **Expected Behavior**: Clicking the delete button (trash icon) on any project card or menu opens an explicit modal confirmation dialog displaying the full file path. Confirming permanently deletes the `.siro` file and any `.siro.bak` backup from disk.
- **Rules & Invariants**: Siro implements permanent deletion without leaving orphan files. The deleted path is atomically pruned from `open_projects`, `recent_files`, and `starred_files`. If the deleted project is currently open in a tab, its tab is closed immediately without prompting to save.

#### 32. Authoritative In-App Project & File Renaming
A synchronized renaming system that keeps memory, UI, and filesystem in total agreement.
- **Expected Behavior**: Users can rename a project by clicking its title in the top navigation bar or through the file menu. Renaming updates the active window title, tab title, and manifest metadata immediately.
- **Rules & Invariants**: If the project has an existing disk file, `SiroApp::rename_project` performs an atomic filesystem rename of the `.siro` file and `.siro.bak` file, handles NTFS case-only renames via temporary file, sanitizes invalid characters, and updates all recent and starred references.

---

### 5.5. User Interface, Theming & Controls

#### 33. Miro-Style Floating Top Navigation Bar
The top-level application header combining branding, tabs, project status, and window controls into a unified bar.
- **Expected Behavior**: Hosts the Siro logo mark, project tab strip, inline editable project title, status capsule (`"Local • Saved"` / `"Local • Edited"`), hamburger file menu, undo/redo buttons, zoom level readout, export dropdown, and theme switch.
- **Rules & Invariants**: Never truncates tab titles abruptly; handles multi-tab overflow gracefully. Status pill is purely informative and click-to-save; contains no fake dropdown chevrons.

#### 34. Persistent Dual-Theme Engine (Light & Dark Modes)
A complete visual theming system supporting both high-contrast Dark Mode and clean engineering Light Mode.
- **Expected Behavior**: Clicking the theme button in the top bar or menu switches the entire visual presentation (canvases, sidebars, cards, properties bars, fonts, and grid lines).
- **Rules & Invariants**: Theme choice is saved atomically to `%LOCALAPPDATA%\Siro\theme.json` and restored on application startup. Color tokens maintain accessible contrast ratios across both modes.

#### 35. Floating Contextual Properties Inspector Bar
An adaptive floating toolbar that appears directly above selected canvas elements.
- **Expected Behavior**: Contextually changes based on selection:
  - For shapes: stroke width, stroke color palette, fill color palette, corner radius.
  - For text: font size, font style, alignment, text background color, opacity.
  - For tables: cell background colors, column width controls, row actions.
  - For electronics: component value edit, pin labeling, rotation button.
  - For images: aspect ratio lock, perspective reset, original size reset.
- **Rules & Invariants**: The properties bar renders in canvas overlay coordinates above the bounding box. It disappears automatically when selection is cleared. Adjusting properties registers an undo step and calls `project.mark_edited()`.

#### 36. Curated STEM & Hardware Color Preset Palettes
A purpose-designed color system tailored for hardware schematics and technical diagramming.
- **Expected Behavior**: Color pickers provide quick-select circles for:
  - **Brand Blue** (`#2563eb`): Primary logic and structural elements.
  - **Signal Red** (`#dc2626`): Power lines, positive voltage, warnings, alarms.
  - **Ground Black / Slate** (`#0f172a`): Ground nets, return paths, structural borders.
  - **VCC Amber** (`#d97706`): Intermediate voltages, clock signals, pull-ups.
  - **Bus Green** (`#16a34a`): Data buses, verified nets, GPIO pins.
  - **Wire Gray** (`#64748b`): Passive connections, neutral annotations.
  - **Accent Purple** (`#9333ea`): High-voltage, RF traces, special functions.
- **Rules & Invariants**: Clicking a color preset updates the property immediately. Presets include tooltips with functional hardware names rather than generic color labels.

#### 37. Global Keyboard Shortcuts & Conflict Isolation
A comprehensive, conflict-free keyboard shortcut routing system.
- **Expected Behavior**:
  - `V`: Select / Move tool
  - `P`: Pen tool
  - `R`: Shapes tool
  - `T`: Text box tool
  - `W`: Wire tool
  - `E`: Eraser tool
  - `Ctrl+S`: Save project (or Save As if untitled)
  - `Ctrl+O`: Open `.siro` file dialog
  - `Ctrl+N`: New project tab
  - `Ctrl+W`: Close active project tab (with Close Guard protection)
  - `Ctrl+Z`: Undo
  - `Ctrl+Y` / `Ctrl+Shift+Z`: Redo
  - `Ctrl+C` / `Ctrl+V`: Copy and paste selected elements
  - `Ctrl+D`: Duplicate selected elements with offset
  - `Delete`: Delete selected canvas elements
  - `Ctrl + +` / `Ctrl + -`: Zoom in / Zoom out
  - `Ctrl + 0`: Reset zoom to 100%
  - `Space + Drag`: Pan canvas
- **Rules & Invariants**: Single-letter shortcuts (`V`, `P`, `R`, `T`, `W`, `E`) are strictly suppressed when the user is typing inside an active text input or table cell to prevent accidental tool switching during text entry.

---

## 6. Deep-Dive: Native `.siro` File Format & Storage Engine

### 6.1. Archive Anatomy & Compression
A `.siro` file is a standard, unencrypted PKZIP archive compressed with the standard Deflate algorithm. Renaming `project.siro` to `project.zip` allows inspecting its full contents with standard archiving tools (7-Zip, WinRAR, Python `zipfile`, or the OS archive utility).

```text
my_hardware_board.siro (ZIP Archive - Deflate)
├── manifest.json                     # Project header, version, ordered board UUID list
├── boards/                           # Directory of canvas sheets (one JSON per sheet)
│   ├── e25c2793-....json             # Sheet 1: Elements, camera position, background style
│   └── 8a1f49b1-....json             # Sheet 2: Elements, camera position, background style
├── planner.json                      # Project planner: Kanban goals/tasks + Bill of Materials
└── assets/                           # Embedded binary payloads (images, datasheets, PDFs)
    ├── 3c49a1d2-..._schematic.png    # Raw image bytes
    └── 7f20b4a8-..._esp32_pinout.png # Raw image bytes
```

### 6.2. JSON Schemas & File Payloads

#### `manifest.json`
Located at the root of the archive. Contains top-level metadata:
```json
{
  "name": "ESP32 Sensor Hub",
  "version": "0.1.0",
  "board_order": [
    "e25c2793-1823-4556-9a22-3510e1136b80",
    "8a1f49b1-7890-4221-a123-567890abcdef"
  ],
  "active_board": "e25c2793-1823-4556-9a22-3510e1136b80",
  "created_at": "2026-10-28T14:32:00Z",
  "updated_at": "2026-10-31T09:15:22Z"
}
```

#### `boards/{board_id}.json`
Represents an individual canvas sheet. Each element in `elements` is serialized as a tagged enum variant corresponding to `BoardElement`:
```json
{
  "id": "e25c2793-1823-4556-9a22-3510e1136b80",
  "name": "Power & MCU Schematic",
  "camera_pan": [120.5, -45.0],
  "camera_zoom": 1.25,
  "created_at": "2026-10-28T14:32:00Z",
  "updated_at": "2026-10-31T09:15:22Z",
  "elements": [
    {
      "Component": {
        "id": "11111111-2222-3333-4444-555555555555",
        "kind": "Resistor",
        "pos": [200.0, 300.0],
        "rotation_quarters": 0,
        "designator": "R1",
        "value": "10kΩ",
        "terminals": [
          { "id": "aaaa...", "pos": [180.0, 300.0], "name": "1" },
          { "id": "bbbb...", "pos": [220.0, 300.0], "name": "2" }
        ],
        "width": 60.0,
        "height": 24.0
      }
    },
    {
      "Image": {
        "id": "22222222-3333-4444-5555-666666666666",
        "asset_id": "3c49a1d2-0000-1111-2222-333344445555",
        "image_name": "schematic.png",
        "pos": [400.0, 250.0],
        "size": [320.0, 240.0],
        "opacity": 1.0,
        "perspective_locked": true,
        "corners": null
      }
    }
  ]
}
```

#### `planner.json`
Serializes project milestones, goals, and the Bill of Materials (BOM):
```json
{
  "goals": [
    {
      "id": "55555555-6666-7777-8888-999999999999",
      "title": "Complete Power Regulation Schematic",
      "completed": true,
      "is_hackclub_milestone": false
    }
  ],
  "bom_items": [
    {
      "id": "66666666-7777-8888-9999-000000000000",
      "name": "Resistor 10k 0805",
      "part_number": "RC0805FR-0710KL",
      "spec_value": "10kΩ",
      "quantity": 4,
      "unit_cost": 0.05,
      "acquired": true,
      "supplier_url": "https://lcsc.com/...",
      "footprint_or_notes": "0805 SMD"
    }
  ],
  "hackclub_info": {
    "project_name": "ESP32 Sensor Hub",
    "ship_status": "Prototyping",
    "update_url": "",
    "slack_channel": "#proj-sensor-hub",
    "arcade_hours": 12.5,
    "grant_budget": 100.0
  }
}
```

#### `assets/` Directory
Binary assets (imported PNG/JPEG images, PDF docs) are stored directly inside the ZIP under `assets/<uuid>_<original_filename>`. The model references assets strictly by their UUID. When deserializing:
- Reading extracts raw bytes into the in-memory `AssetStore`.
- Serializing streams bytes from `AssetStore` directly to the archive without Base64 overhead.

### 6.3. Persistence Workflow & Atomic Guarantee
Every write operation performed by `SiroFileManager::save_project` guarantees zero data corruption:
1. **Directory Verification**: Validates parent folder presence; creates directories if absent.
2. **Staging File**: Writes to a temporary hidden file in the same directory: `.<filename>.tmp_<uuid>`. Writing to the same directory ensures atomic rename works across filesystem boundaries.
3. **Flushed Zip Stream**: The ZIP container is written and explicitly flushed (`zip.finish()`), releasing file locks.
4. **Safety Backup**: If the destination `.siro` already exists, it is duplicated to `<destination>.siro.bak`.
5. **Atomic Rename / Replace**: Replaces target via `std::fs::rename`. On Windows NTFS where active handles or locks can prevent direct overwriting, it safely deletes the destination prior to rename.
6. **Revision Sync**: Once the filesystem rename succeeds, `project.filepath` is updated and `project.mark_saved()` synchronizes `saved_revision = edit_revision`.

---

## 7. Application Execution Lifecycle & Frame Loop

Siro runs on an immediate-mode GUI architecture (`eframe::App::update`). The sequence of operations executed on every frame is deterministic:

```text
                               ┌────────────────────────────────┐
                               │       Window Frame Start       │
                               └────────────────┬───────────────┘
                                                │
                                                ▼
                        ┌───────────────────────────────────────────────┐
                        │ 1. Window Close Interception                  │
                        │    ctx.input(|i| i.viewport().close_req())    │
                        │    If unsaved & !allow_close:                 │
                        │       CancelClose + Open Modal Dialog         │
                        └───────────────────────┬───────────────────────┘
                                                │
                                                ▼
                        ┌───────────────────────────────────────────────┐
                        │ 2. Global Keyboard Shortcuts                  │
                        │    shortcuts.rs: Undo, Redo, Save, Tools      │
                        │    (Suppressed when typing in text fields)    │
                        └───────────────────────┬───────────────────────┘
                                                │
                                                ▼
                        ┌───────────────────────────────────────────────┐
                        │ 3. View Routing (AppState)                    │
                        │    ├── AppState::Home                         │
                        │    │   Render Sidebar, Templates, Recents     │
                        │    └── AppState::Editor                       │
                        │        Render Top Bar, Multi-Project Tabs     │
                        │        Render Left Tool Rail                  │
                        │        Render 2D Canvas (Frustum Culled)      │
                        │        Render Floating Properties Inspector   │
                        │        Render Right Planner Drawer            │
                        └───────────────────────┬───────────────────────┘
                                                │
                                                ▼
                        ┌───────────────────────────────────────────────┐
                        │ 4. Modal Dialog Layer                         │
                        │    Close Guard Dialog / Delete Confirm Modal  │
                        └───────────────────────┬───────────────────────┘
                                                │
                                                ▼
                        ┌───────────────────────────────────────────────┐
                        │ 5. End-of-Frame Autosave Sweep                │
                        │    AutosaveManager::tick(&mut projects)       │
                        │    Saves eligible dirty projects silently     │
                        └───────────────────────┬───────────────────────┘
                                                │
                                                ▼
                               ┌────────────────────────────────┐
                               │   Frame Commit (wgpu Render)   │
                               └────────────────────────────────┘
```

---

## 8. Key Subsystems & Architectural Invariants

### 8.1. Revision-Counter Dirty State
Sticky boolean flags (`Project.dirty: bool`) have been eliminated. Monotonic revision counters provide strict data integrity:
- `edit_revision: u64`: Incremented via `project.mark_edited()` whenever modifications occur.
- `saved_revision: u64`: Updated to `edit_revision` via `project.mark_saved()` strictly after successful disk persistence.
- `is_unsaved(&Project) -> bool`: Evaluates `edit_revision != saved_revision`.
- `has_content(&Project) -> bool`: Returns `true` if any sheet has elements, multiple sheets exist, sheets were renamed from default, assets exist, or planner tasks/BOM differ from default template.

### 8.2. Close Guard & Exit Interception
All exit paths converge on the pure `close_plan()` decision function (`src/ui/close_dialog.rs`):
- **Window Close (OS Button, Taskbar, Alt+F4)**: Intercepted in `src/app.rs` via `ctx.input(|i| i.viewport().close_requested())`. Emits `CancelClose` and displays the modal `CloseDialogState`.
- **Tab Close**: `request_close_tab(idx)` prompts modal dialog if dirty, or closes immediately if pristine.

### 8.3. Project File Format (`.siro`)
A `.siro` file is an unencrypted ZIP archive containing `manifest.json`, `board_<id>.json`, `planner.json`, and an `assets/` folder. All saves write to temporary staging files first.

### 8.4. AppSettings & Autosave Engine
- **Settings Path**: `%LOCALAPPDATA%\Siro\settings.json` (or `~/.siro/settings.json`).
- **Schema**: `{"autosave_interval_secs": 60}` (allowed: `5..=600` seconds; `null` = disabled).
- **Sweep Mechanism**: At the end of every frame in `src/app.rs`, `AutosaveManager::tick(&mut self.open_projects, &self.settings)` sweeps open projects with a known disk `filepath` and unsaved changes.

### 8.5. Project Naming & Renaming Invariants
- **Display Name**: Derived authoritatively via `Project::display_name(&self) -> &str`.
- **Renaming**: Renaming a project invokes `SiroApp::rename_project()`, which synchronizes disk files (`.siro` and `.siro.bak`), handles NTFS case-only renames, updates recent/starred trackers, and marks the project saved.

### 8.6. UI Honesty Principles
- Zero fake collaboration UI, fake workspaces, fake user avatars, or dead dropdown chevrons.
- Recent files show honest file size in KB/MB and actual disk modified timestamps.
- Dark Mode toggle persists atomically across restarts in `%LOCALAPPDATA%\Siro\theme.json`.

---

## 9. Developer Extensibility Cookbooks

These recipes demonstrate how to implement new functionality in Siro across the codebase without violating core invariants.

### Cookbook A: Adding a New Electronic Component
1. **Define Kind**: In `src/model/electronics.rs`, add the variant to `ComponentKind` (e.g. `Potentiometer`).
2. **Terminal Pins & Dimensions**: In the same file, implement `default_size()`, `default_designator()` (e.g. `"POT"`), and `generate_terminals()` to define the local $(x,y)$ coordinates of each pin.
3. **Renderer**: In `src/canvas/render.rs`, add a match arm in component rendering to draw the schematic symbol via `painter.line_segment()` or `painter.circle_stroke()`.
4. **Tool Rail Catalog**: In `src/ui/tool_rail.rs`, add the component button to the electronics flyout menu with its display label and icon.

### Cookbook B: Adding a New Canvas Drawing Tool
1. **Register Tool**: In `src/canvas/tools/mod.rs`, add the tool variant to `pub enum CanvasTool` (e.g. `Arrow`).
2. **Implement Logic**: Create `src/canvas/tools/arrow.rs` implementing tool event handling (`on_pointer_down`, `on_pointer_moved`, `on_pointer_up`).
3. **Connect Tool Rail**: In `src/ui/tool_rail.rs`, add the icon and selection click handler.
4. **Shortcut**: In `src/shortcuts.rs`, map the keyboard shortcut (e.g. `A`) and add to shortcut help labels.

### Cookbook C: Adding a New Persistent Setting
1. **Extend Struct**: In `src/storage/settings.rs`, add the new field to `pub struct AppSettings` with a `#[serde(default = "...")]` fallback function.
2. **Settings UI**: In `src/ui/home_screen.rs`, locate `render_settings_view()` and add the UI control (e.g. slider, checkbox, or combo box).
3. **Save Handler**: When modified in UI, invoke `app_settings.save()`.

---

## 10. Local Storage & Environment Paths

Siro stores all user data locally on the host machine. The exact paths used across platforms:

| Identifier | Windows Path | macOS / Linux Fallback | Description |
| :--- | :--- | :--- | :--- |
| **Settings** | `%LOCALAPPDATA%\Siro\settings.json` | `~/.siro/settings.json` | Autosave interval and preferences |
| **Theme** | `%LOCALAPPDATA%\Siro\theme.json` | `~/.siro/theme.json` | Dark Mode state (`{"dark_mode": true}`) |
| **Recent Files** | `%LOCALAPPDATA%\Siro\recent_files.json` | `~/.siro/recent_files.json` | LRU list of recently opened project paths |
| **Starred Files** | `%LOCALAPPDATA%\Siro\starred_files.json` | `~/.siro/starred_files.json` | Pinned favorite project paths |
| **Safety Backups**| `<dir>\<name>.siro.bak` | `<dir>/<name>.siro.bak` | Automatic backup alongside original file |
| **Attachments** | `%LOCALAPPDATA%\Siro\attachments\` | `~/.siro/attachments/` | Extracted doc attachments cache |

---

## 11. Testing & Quality Assurance

The codebase includes an extensive automated test suite covering all core business logic (119 total tests):

| Test File | Focus Area |
| :--- | :--- |
| `tests/autosave_and_close_tests.rs` | Revision counters, `close_plan()` matrix, `AppSettings` atomic persistence |
| `tests/canvas_tests.rs` | UI honesty, branding enforcement, theme persistence, recents metadata, board switching |
| `tests/new_features_tests.rs` | Data table tools, electronics models, sheet duplicate/reorder ops |
| `tests/snap_tests.rs` | Bounding box edge alignment, center snapping, grid snapping math |
| `tests/undo_and_model_tests.rs` | Undo/redo stack integrity, element mutations, serialization roundtrips |

Run all tests anytime with:
```powershell
cargo test
```

---

## 12. Guidelines for AI Agents & Future Contributors

1. **Keep Single-User Offline Focus**: Do not add remote collaboration, web sockets, fake user accounts, or cloud sync mockups unless an actual local backend is implemented.
2. **Preserve Revision Invariants**: When modifying project state or adding new tools, always invoke `project.mark_edited()`. Never re-introduce a standalone `dirty: bool`.
3. **Verify Line by Line**: Before committing or concluding tasks, run `cargo check --all-targets` and `cargo test` to ensure zero compilation warnings or test regressions.
4. **Maintain .gitignore Cleanliness**: Only track project source files, tests, assets, installer scripts, and this master context file (`PROJECT_CONTEXT.md`). Do not track ephemeral audit logs, session dumps, or build artifacts.
