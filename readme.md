# Flambé

[![license](https://img.shields.io/badge/license-GPLv3-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/flambe.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/flambe.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**Flambé** — A PC visual editor for game-development-oriented project files compatible with the Alight Motion format.

| English | Simplified Chinese            |
|---------|-------------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`Flambé` is a desktop editor built with [Rust](https://www.rust-lang.org/) and [Bevy](https://bevyengine.org/), designed for editing `.amproj`-compatible project files within a game development context.

It provides a familiar timeline-based editing workflow with real-time SDF rendering, keyframe animation, and an egui-powered editor UI — all running on the Bevy game engine.

**This tool is specifically built for game development workflows** and is not intended as a general-purpose motion graphics application.

## Features

* **Project file I/O** — Read and write `.amproj`-compatible project files with full round-trip fidelity
* **Real-time SDF preview** — Live rendering of shapes, text, and effects via the Bevy engine
* **Timeline editor** — AE-style unified layer + timeline panel with keyframe visualization
* **Property inspector** — View and edit layer transforms, effects, and metadata
* **Playback controls** — Play, pause, seek, loop, and speed control
* **CJK font support** — Automatic system CJK font detection for multilingual UI
* (Planned) **Keyframe editing** — Add, delete, and move keyframes on the timeline
* (Planned) **Save workflow** — Export modified projects back to `.amproj` format
* (Planned) **Undo/Redo** — Full command-based undo system

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/Bli-AIk/flambe.git
   cd flambe
   ```

3. **Build and run**:
   ```bash
   cargo run
   ```

4. **Open a project**:
   * Use **File > Open...** to select a `.amproj` file
   * The project will load with full layer hierarchy and timeline display
   * Use the playback controls to preview animations

## How to Build

### Prerequisites

* Rust 1.85 or later
* System dependencies (Linux):
  ```bash
  sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev \
      libwayland-dev libxkbcommon-dev
  ```

### Build Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Bli-AIk/flambe.git
   cd flambe
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

## Dependencies

This project uses the following main crates:

| Crate                                                                     | Version | Description                        |
|---------------------------------------------------------------------------|---------|------------------------------------|
| [bevy](https://crates.io/crates/bevy)                                    | 0.18    | Game engine and ECS framework      |
| [bevy_alight_motion](https://github.com/Bli-AIk/bevy_alight_motion)      | (path)  | AM project loader and SDF renderer |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)      | 0.36    | egui integration for Bevy          |
| [egui](https://crates.io/crates/egui)                                    | 0.33    | Immediate mode GUI library         |
| [quick-xml](https://crates.io/crates/quick-xml)                          | 0.39    | XML serialization/deserialization  |
| [zip](https://crates.io/crates/zip)                                      | 7.4     | ZIP archive handling               |
| [rfd](https://crates.io/crates/rfd)                                      | 0.15    | Native file dialogs                |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under the **GNU General Public License v3.0** — see the [LICENSE](LICENSE) file for details.

This means you are free to use, modify, and distribute this software, provided that any derivative works are also distributed under the same license terms. Commercial closed-source use is not permitted.
