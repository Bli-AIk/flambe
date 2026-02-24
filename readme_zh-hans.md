# Flambé

[![license](https://img.shields.io/badge/license-GPLv3-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/flambe.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/flambe.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" />

> 当前状态：🚧 早期开发中（初始版本进行中）

**Flambé** — 面向游戏开发的 PC 端可视化编辑器，用于编辑兼容 Alight Motion 格式的工程文件。

| English                | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 简介

`Flambé` 是一个使用 [Rust](https://www.rust-lang.org/) 和 [Bevy](https://bevyengine.org/) 构建的桌面编辑器，专为在游戏开发环境中编辑 `.amproj` 兼容工程文件而设计。

它提供了基于时间轴的编辑工作流，具有实时 SDF 渲染、关键帧动画和基于 egui 的编辑器界面——全部运行在 Bevy 游戏引擎之上。

**此工具专为游戏开发工作流构建**，并非通用动态图形应用程序。

## 功能特性

* **工程文件读写** — 读写 `.amproj` 兼容工程文件，支持完整的往返保真度
* **实时 SDF 预览** — 通过 Bevy 引擎实时渲染形状、文本和效果
* **时间轴编辑器** — AE 风格的统一图层 + 时间轴面板，带关键帧可视化
* **属性检查器** — 查看和编辑图层变换、效果及元数据
* **播放控制** — 播放、暂停、定位、循环和速度控制
* **CJK 字体支持** — 自动检测系统 CJK 字体，实现多语言 UI
* （计划中）**关键帧编辑** — 在时间轴上添加、删除和移动关键帧
* （计划中）**保存工作流** — 将修改后的工程导出为 `.amproj` 格式
* （计划中）**撤销/重做** — 完整的基于命令的撤销系统

## 使用方法

1. **安装 Rust**（如未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆仓库**：
   ```bash
   git clone https://github.com/Bli-AIk/flambe.git
   cd flambe
   ```

3. **构建并运行**：
   ```bash
   cargo run
   ```

4. **打开工程**：
   * 使用 **File > Open...** 选择 `.amproj` 文件
   * 工程将加载完整的图层层级和时间轴显示
   * 使用播放控件预览动画

## 构建方法

### 前置要求

* Rust 1.85 或更高版本
* 系统依赖（Linux）：
  ```bash
  sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev \
      libwayland-dev libxkbcommon-dev
  ```

### 构建步骤

1. **克隆仓库**：
   ```bash
   git clone https://github.com/Bli-AIk/flambe.git
   cd flambe
   ```

2. **构建项目**：
   ```bash
   cargo build --release
   ```

3. **运行测试**：
   ```bash
   cargo test
   ```

## 依赖

本项目使用以下主要 crate：

| Crate                                                                     | 版本    | 说明                    |
|---------------------------------------------------------------------------|---------|------------------------|
| [bevy](https://crates.io/crates/bevy)                                    | 0.18    | 游戏引擎和 ECS 框架     |
| [bevy_alight_motion](https://github.com/Bli-AIk/bevy_alight_motion)      | (path)  | AM 工程加载器和 SDF 渲染 |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)      | 0.36    | Bevy 的 egui 集成       |
| [egui](https://crates.io/crates/egui)                                    | 0.33    | 即时模式 GUI 库         |
| [quick-xml](https://crates.io/crates/quick-xml)                          | 0.39    | XML 序列化/反序列化     |
| [zip](https://crates.io/crates/zip)                                      | 7.4     | ZIP 压缩包处理          |
| [rfd](https://crates.io/crates/rfd)                                      | 0.15    | 原生文件对话框          |

## 贡献

欢迎贡献！无论是修复 bug、添加功能还是改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目采用 **GNU 通用公共许可证 v3.0** 授权 — 详见 [LICENSE](LICENSE) 文件。

这意味着您可以自由使用、修改和分发本软件，前提是任何衍生作品也必须在相同的许可证条款下分发。不允许商业闭源使用。
