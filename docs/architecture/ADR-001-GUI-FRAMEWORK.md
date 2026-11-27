# ADR-001: Rust-Native Desktop GUI Framework Selection

- **Status:** Accepted
- **Date:** 2026-11-27
- **Deciders:** Keeftalk Desktop Engineering

---

## Context

Keeftalk Desktop is a cross-platform (Windows 10/11, Linux, macOS) privacy-first Super App written in Rust. We must select a native Rust GUI framework without relying on Electron, Node.js, or bundled WebViews.

The UI must support:
1. Dense multi-module desktop layouts (sidebar, command palette, split panes, chat threads, Markdown note editor, calendar grid, encrypted vault file browser, charts/heatmaps).
2. Strict separation of concerns: the UI layer must never touch SQLite, cryptographic keys, or HTTP/WebSocket clients directly.
3. Multilingual text shaping and bidirectional layout (English, French, Arabic as present in Android `res/values-*`).
4. Custom theming (Light, Dark, Day, Pinky, Masculine, configurable accent colors, and Keef glassmorphic/atmospheric tokens).
5. High-DPI fractional scaling, keyboard navigation, and GPU acceleration with software fallback.
6. OSI-approved open-source licensing compatible with `GPL-3.0-or-later`.

---

## Candidates Evaluated

### 1. Iced (`iced`)
- **Architecture:** Pure-Rust Elm Architecture (Model-View-Update / TEA) with async `Task` commands and `Subscription` streams running cleanly on Tokio.
- **Rendering:** `wgpu` (Vulkan / DX12 / Metal / OpenGL) with automatic `tiny-skia` software rendering fallback on VMs, headless runners, or older GPUs.
- **Text & Internationalization:** Uses `cosmic-text`, providing native HarfBuzz/Unicode shaping, BiDi/RTL support (Arabic), emoji fallback, and a built-in `text_editor` widget plus native `markdown` widget.
- **Custom Graphics:** Built-in `canvas` (vector drawing for habit heatmaps, market charts, calendar timelines) and `svg` widget (renders Keeftalk's official SVG logo cleanly at any DPI).
- **Cross-Platform & Dependencies:** Pure Rust; compiles cleanly on Windows, Linux (Wayland & X11), and macOS with minimal system libraries.
- **License:** MIT (maximally permissive; fully compatible with GPL-3.0-or-later).

### 2. Slint (`slint`)
- **Architecture:** Declarative `.slint` UI markup compiled to Rust callbacks/properties.
- **Strengths:** Polished declarative styling, strong embedded/desktop accessibility, good performance.
- **Weaknesses for Keeftalk:**
  - Dual-licensed (GPLv3 or commercial); while compatible with GPLv3, `.slint` DSL creates a rigid boundary for highly dynamic Super App views (rich-text/Markdown live preview, arbitrary split panes, custom canvas charts, dynamic command palettes, live video frame rendering).
  - Less idiomatic integration with pure-Rust async state machines across 19 modules compared to Iced's TEA `Message`/`Task` model.

### 3. egui (`eframe`)
- **Architecture:** Immediate-mode GUI.
- **Strengths:** Fast iteration, simple custom widgets.
- **Weaknesses for Keeftalk:** Weak complex script / RTL text shaping (Arabic), non-native layout ergonomics for dense text/productivity apps, and higher idle redraw complexity without careful retention optimization.

### 4. gtk4-rs / libadwaita
- **Strengths:** Mature Linux accessibility and native GNOME feel.
- **Weaknesses for Keeftalk:** Heavy C dependency tree on Windows and macOS (`GTK4` + `Pango` + `Cairo` DLLs/frameworks), non-native look on Windows/macOS, and complex cross-platform packaging.

### 5. Tauri / Dioxus WebView
- **Rejected:** Relies on OS WebView / HTML-CSS-JS runtime, conflicting with the Rust-native GUI requirement.

---

## Decision Matrix

| Criterion | Iced | Slint | egui | gtk4-rs |
|---|---|---|---|---|
| Rust-native (no WebView) | Yes | Yes | Yes | Bindings |
| Architecture fit (strict UI/Core separation) | **Excellent (TEA)** | Good | Fair | Fair |
| Cross-platform packaging (Win/Linux/macOS) | **Excellent** | Excellent | Excellent | Poor on Win/Mac |
| RTL / Arabic / French / Unicode shaping | **Excellent (`cosmic-text`)** | Good | Limited | Excellent |
| Custom Canvas / SVG / Markdown / Editor | **Built-in** | Partial | Manual | Manual |
| License | **MIT** | GPLv3 / Commercial | MIT/Apache-2.0 | LGPL-2.1 |
| Long-term community momentum | Very High | High | High | High (Linux) |

---

## Decision

We select **Iced** as the GUI framework for Keeftalk Desktop (`crates/keeftalk-ui`).

### Consequences
- All UI state lives in `keeftalk-ui` view models and renders purely from immutable state snapshots.
- Side effects (database reads/writes, crypto operations, Supabase HTTP/Realtime calls) execute in `keeftalk-core` / service crates via Tokio and communicate with `keeftalk-ui` exclusively through typed messages (`Task` / `Subscription`).
- `wgpu` + `tiny-skia` dual backend guarantees Keeftalk runs on both modern GPUs and software-rendered environments.
