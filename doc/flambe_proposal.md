# Flambé — PC 端 Alight Motion 可视化编辑器

> **Flambé**（法式火焰烹饪）— 基于 bevy_alight_motion 和 egui 的 .amproj 可视化编辑器。

## 1. 项目目标

构建一个 **最小可行性产品 (MVP)**，为 bevy_alight_motion 已实现的功能提供完整的可视化编辑能力。

**核心原则**：

- 编辑器编辑/导出的仍然是标准 `.amproj` 文件
- 编辑器产出的 `.amproj` 必须在 Android 端 Alight Motion 中正确打开和编辑
- 不制作任何 PC 端独有功能，与 AM 保持完全兼容
- bevy_alight_motion + Bevy 作为渲染引擎，egui 作为 UI 层

---

## 2. 可行性分析

### 2.1 已有基础（强项）

| 模块            | 现状                                     | 可复用程度 |
|---------------|----------------------------------------|-------|
| **amproj 解析** | 完整的 XML 反序列化，支持所有图层类型                  | ★★★★★ |
| **动画系统**      | 完整的关键帧插值 + 8 种缓动函数                     | ★★★★★ |
| **渲染管线**      | SDF 形状、Sprite、RTT、UnifiedEffect        | ★★★★★ |
| **效果系统**      | 15+ 种效果（Transform2、Wipe、Blur、Repeat 等） | ★★★★☆ |
| **场景管理**      | 图层层级、嵌套编组、生命周期、坐标转换                    | ★★★★★ |
| **效果注册表**     | 完整的效果定义、字段类型、实现状态扫描                    | ★★★★☆ |
| **验证系统**      | 自动检测支持/不支持的特性                          | ★★★★☆ |

### 2.2 需要新增的能力

| 模块              | 必要性   | 难度    | 说明                                 |
|-----------------|-------|-------|------------------------------------|
| **XML 序列化（写回）** | ★★★★★ | ★★★★☆ | ✅ 已为所有类型添加 Serialize + 自定义序列化器 |
| **时间轴 UI**      | ★★★★★ | ★★★★★ | egui 定制：轨道、关键帧、拖拽、缩放、吸附            |
| **属性面板**        | ★★★★★ | ★★★☆☆ | egui 表单：变换、颜色、效果参数                 |
| **缓动曲线编辑器**     | ★★★★★ | ★★★★☆ | 可视化贝塞尔曲线编辑（Bezier 手柄拖拽）            |
| **图层管理**        | ★★★★☆ | ★★★☆☆ | 添加/删除/重排/分组                        |
| **撤销/重做**       | ★★★★☆ | ★★★☆☆ | 命令模式                               |
| **实时预览同步**      | ★★★★★ | ★★★☆☆ | 编辑器操作 → 实时更新 Bevy ECS              |

### 2.3 关键挑战

#### 挑战 1：Round-trip 保真（XML 序列化）

**问题**：当前 `schema::types.rs` 使用 `serde::Deserialize` 进行反序列化，但没有 `Serialize`。更关键的是，AM 的 XML
格式包含许多我们目前**解析但不使用**或**完全忽略**的字段和属性。如果我们仅序列化已解析的字段，写回的 `.amproj` 将丢失信息。

**方案**：

- **推荐方案**：在加载时保留原始 XML DOM（或至少保留未识别属性的 raw map），写回时将编辑后的结构合并回原始 DOM
- **备选方案**：为所有 schema 类型添加 `#[serde(flatten)] pub extra: HashMap<String, serde_json::Value>` 捕获未识别字段
- 需要编写全面的 round-trip 测试：`load → save → load` 对比结果一致

**风险评估**：中高风险。AM 的 XML 格式并非完全文档化，可能存在隐含的字段顺序依赖或特殊格式要求。需要大量兼容性测试。

#### 挑战 2：时间轴 UI 复杂度

**问题**：egui 是 immediate-mode GUI，不是为复杂时间轴编辑设计的。需要实现：

- 多轨道水平时间轴（类似 After Effects / AM 的时间轴）
- 关键帧钻石标记的精确拖拽
- 时间游标（playhead）的拖拽和吸附
- 轨道的垂直折叠/展开
- 缩放和平移（时间轴的 zoom/pan）

**方案**：

- 使用 egui 的 `Painter` API 进行完全自定义绘制
- 参考现有 egui 时间轴实现（如 `egui_timeline`、`egui_animation`）
- 事件处理通过 egui 的 `Response` API 实现交互

**风险评估**：高风险。这是编辑器中最复杂的 UI 组件，直接决定用户体验。

#### 挑战 3：实时预览与编辑同步

**问题**：bevy_alight_motion 的 ECS 结构是为播放设计的：加载 amproj → 生成 ECS 实体 →
播放。编辑器需要反向操作：用户编辑参数 → 实时更新 ECS。

**方案**：

- **编辑器数据模型**（`EditorProject`）作为 single source of truth
- 用户编辑 → 更新 `EditorProject` → 差量更新 Bevy ECS 组件
- 不需要每次都重新加载 amproj，仅 patch 变更的组件
- `AmAnimated` 组件的字段直接对应可编辑的属性，修改可直接反映

**风险评估**：中风险。需要仔细设计数据流，但架构上可行。

---

## 3. 架构设计

### 3.1 整体架构

```
┌──────────────────────────────────────────────────────────┐
│                     Flambé Application                    │
├──────────────────┬───────────────────────────────────────┤
│   egui UI Layer  │         Bevy Render Viewport          │
│                  │                                       │
│  ┌────────────┐  │  ┌─────────────────────────────────┐  │
│  │ 菜单栏      │  │  │                                 │  │
│  ├────────────┤  │  │    bevy_alight_motion 实时渲染     │  │
│  │ 图层面板    │  │  │    (SDF + Sprite + RTT)          │  │
│  ├────────────┤  │  │                                 │  │
│  │ 属性面板    │  │  └─────────────────────────────────┘  │
│  ├────────────┤  │                                       │
│  │ 效果面板    │  │  ┌─────────────────────────────────┐  │
│  └────────────┘  │  │          时间轴面板                │  │
│                  │  │  [Track 1] ◆──────◆──────◆       │  │
│                  │  │  [Track 2]    ◆────────◆         │  │
│                  │  │  [Track 3] ◆──◆                  │  │
│                  │  │          ▼ (playhead)             │  │
│                  │  └─────────────────────────────────┘  │
├──────────────────┴───────────────────────────────────────┤
│                    Core Data Layer                        │
│  ┌─────────────┐  ┌──────────┐  ┌─────────────────────┐ │
│  │EditorProject│  │Undo/Redo │  │  amproj I/O         │ │
│  │ (数据模型)   │  │  Stack   │  │ (XML 读写)           │ │
│  └─────────────┘  └──────────┘  └─────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

### 3.2 Crate 结构

```
crates/
  flambe/                          # 主编辑器 crate
    src/
      lib.rs
      app.rs                       # Bevy App 入口
      editor/
        mod.rs
        project.rs                 # EditorProject 数据模型
        selection.rs               # 选中状态管理
        commands.rs                # 可撤销命令（Command Pattern）
        history.rs                 # Undo/Redo 栈
        clipboard.rs               # 复制/粘贴
      ui/
        mod.rs
        menu_bar.rs                # 顶部菜单
        layer_panel.rs             # 图层列表面板
        property_panel.rs          # 属性编辑面板
        effect_panel.rs            # 效果管理面板
        timeline/
          mod.rs
          timeline_widget.rs       # 时间轴主组件
          track.rs                 # 轨道渲染
          keyframe.rs              # 关键帧标记
          playhead.rs              # 播放头
          easing_editor.rs         # 缓动曲线编辑器
        viewport.rs                # Bevy 渲染视口控制
      io/
        mod.rs
        serializer.rs              # AmScene → XML 序列化
        amproj_writer.rs           # ZIP 打包写回
        round_trip.rs              # Round-trip 保真测试工具
      sync/
        mod.rs
        editor_to_ecs.rs           # EditorProject → Bevy ECS 同步
        ecs_to_editor.rs           # 初始加载：ECS → EditorProject
    Cargo.toml
```

### 3.3 核心数据流

```
用户操作（egui）
    │
    ▼
EditCommand（可撤销命令）
    │
    ├──► EditorProject（数据模型更新）
    │         │
    │         ├──► Bevy ECS 差量更新（实时预览）
    │         │
    │         └──► 标记"未保存"
    │
    └──► Undo/Redo Stack（历史记录）

保存操作：
    EditorProject → AmScene (schema) → XML → ZIP (.amproj)
```

### 3.4 依赖关系

```toml
[dependencies]
bevy = { version = "0.18", features = ["jpeg"] }
bevy_alight_motion = { path = "../bevy_alight_motion" }
bevy-inspector-egui = "0.36.0"       # egui integration for Bevy
egui = "0.31"                         # UI framework
quick-xml = { version = "0.39", features = ["serialize"] }  # 需要 serialize feature
zip = { version = "7.4", default-features = false, features = ["deflate"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 4. 功能范围（MVP）

### 4.1 必须实现（P0）

基于 bevy_alight_motion 已实现的全部功能：

**项目管理**：

- [ ] 打开 .amproj 文件
- [ ] 保存 .amproj 文件（round-trip 保真）
- [ ] 项目设置（画布尺寸、FPS、总时长、背景色）

**图层操作**：

- [ ] 图层列表显示（层级缩进、可见性切换）
- [ ] 添加/删除图层（Shape、NullObj、EmbedScene、Text、Image）
- [ ] 图层重排序（拖拽）
- [ ] 图层父子关系设置
- [ ] 图层复制/粘贴

**变换编辑**：

- [ ] 位置 (location) 编辑 + 关键帧
- [ ] 旋转 (rotation) 编辑 + 关键帧
- [ ] 缩放 (scale) 编辑 + 关键帧
- [ ] 不透明度 (opacity) 编辑 + 关键帧
- [ ] 锚点 (pivot) 编辑 + 关键帧

**时间轴**：

- [ ] 多轨道时间轴视图
- [ ] 播放头拖拽
- [ ] 关键帧显示（钻石标记）
- [ ] 关键帧添加/删除/移动
- [ ] 播放/暂停/循环控制
- [ ] 时间轴缩放和平移

**关键帧与缓动**：

- [ ] 关键帧值编辑
- [ ] 缓动类型选择（Linear、Step、CubicBezier、Bounce、ReverseBounce、Cyclic、Elastic、ElasticStep）
- [ ] 贝塞尔缓动曲线可视化编辑器

**效果编辑**（已实现的效果）：

- [ ] Transform2（posx、posy）
- [ ] Wipe2（start、end、angle、feather）
- [ ] Stretch Segment（stretch、angle、offset、smooth）
- [ ] Gaussian Blur（strength）
- [ ] Palette Map（colors、shades、alpha）
- [ ] Replace Color（oldcolor、newcolor、threshold、feather、alpha、lockLuminance）
- [ ] Scale Assist（axis、scale、damp）
- [ ] Repeat（count、offset、angle、scale、alpha）
- [ ] Linear Repeat（全参数）
- [ ] Radial Repeat（全参数）
- [ ] Path Repeat
- [ ] Swing（freq、a1、a2、phase、type）
- [ ] Spin（rpm）
- [ ] 效果添加/删除/重排序

**形状编辑**：

- [ ] 形状类型选择（rect、circle 及所有已实现 SDF 类型）
- [ ] 形状尺寸编辑 + 关键帧
- [ ] 填充类型（颜色/媒体）
- [ ] 填充颜色编辑 + 关键帧
- [ ] 描边宽度/颜色编辑

**实时预览**：

- [ ] 编辑即时同步到 Bevy 渲染视口
- [ ] 视口缩放/平移

### 4.2 应该实现（P1）

- [ ] 撤销/重做（Undo/Redo）
- [ ] 键盘快捷键
- [ ] 多选操作
- [ ] 图层搜索/过滤
- [ ] 形状颜色渐变编辑

### 4.3 可选实现（P2）

- [ ] 嵌入场景（编组）的内部编辑
- [ ] 自定义路径编辑（贝塞尔曲线）
- [ ] 文本编辑（字体选择、大小、对齐）
- [ ] 资源管理器（图片/字体导入）
- [ ] 项目模板

---

## 5. 开发计划

### Phase 1：基础框架 + I/O

**目标**：建立编辑器骨架，实现 amproj 的完整 round-trip I/O。

- [x] 创建 `flambe` crate，配置依赖
- [x] 实现 Bevy + egui 混合渲染窗口布局
- [x] 为 `schema::types.rs` 所有类型添加 `Serialize`
- [x] 实现 `AmScene → XML → ZIP` 序列化流程
- [ ] 编写 round-trip 测试（load → save → load → compare）
- [x] 实现 `EditorProject` 数据模型

### Phase 2：时间轴 + 播放控制

**目标**：实现时间轴核心 UI 和播放控制。

- [x] egui 自定义时间轴组件（轨道 + 关键帧 + 播放头）
- [x] 播放/暂停/循环控制
- [x] 播放头拖拽 → 跳转到指定时间
- [x] 时间轴缩放和平移
- [x] `AmPlayback` 资源与时间轴的双向同步

### Phase 3：图层管理 + 属性面板

**目标**：实现图层列表和基础属性编辑。

- 图层列表面板（层级缩进、可见性、选中高亮）
- 属性面板（变换：位置/旋转/缩放/透明度/锚点）
- EditorProject → Bevy ECS 的差量同步
- 选中状态管理
- 数值编辑 → 实时更新 ECS 组件

### Phase 4：关键帧编辑 + 缓动曲线

**目标**：完整的关键帧编辑能力。

- 关键帧添加/删除/移动（时间轴上拖拽）
- 关键帧值编辑（属性面板输入框）
- 缓动类型下拉选择
- 贝塞尔缓动曲线可视化编辑器（控制点拖拽）
- 缓动预览曲线显示

### Phase 5：效果编辑

**目标**：为所有已实现效果提供编辑 UI。

- 效果面板（当前图层的效果列表）
- 效果添加/删除/重排序
- 各效果的参数编辑表单（基于 effects_registry 的字段定义自动生成）
- 效果参数的关键帧支持

### Phase 6：完善 + 打磨

**目标**：用户体验优化和稳定性。

- 撤销/重做系统
- 键盘快捷键
- 形状编辑（类型切换、描边）
- 多选操作
- 性能优化
- 全面的兼容性测试（在 Android AM 上验证）

---

## 6. 技术决策

### 6.1 为什么选择 egui 而非其他 UI 框架

| 方案           | 优势                                                          | 劣势                    |
|--------------|-------------------------------------------------------------|-----------------------|
| **egui（推荐）** | 已有 bevy 集成（bevy-inspector-egui）、Rust 原生、immediate mode 快速开发 | 时间轴等复杂组件需要大量自定义绘制     |
| Tauri + Web  | 丰富的 UI 组件生态                                                 | 与 Bevy 渲染集成困难，跨进程通信复杂 |
| iced         | Rust 原生 retained mode                                       | Bevy 集成不成熟，组件生态小      |
| bevy_ui      | Bevy 内置                                                     | 功能太基础，不适合复杂编辑器        |

**结论**：egui 是最务实的选择。bevy_alight_motion 已经在 player 示例中使用 `bevy-inspector-egui`，有现成的集成经验。

### 6.2 编辑器数据模型 vs 直接操作 ECS

**选择**：引入独立的 `EditorProject` 数据模型，不直接操作 Bevy ECS。

**理由**：

- ECS 实体的生命周期受图层时间控制（lifecycle system 会 spawn/despawn），不适合作为编辑器的持久数据源
- 撤销/重做需要可序列化的状态快照
- EditorProject 可以直接映射到 AmScene（schema），便于保存

### 6.3 XML 序列化策略

**选择**：`quick-xml` + `serde::Serialize`，辅以原始 XML 保留。

```rust
// 在加载时保存原始 XML 字符串，用于 diff 和 fallback
pub struct EditorProject {
    pub scene: AmScene,           // 结构化数据
    pub original_xml: String,     // 原始 XML（用于 round-trip 对比）
    pub images: HashMap<String, Vec<u8>>,
    pub fonts: HashMap<String, Vec<u8>>,
    // ...
}
```

---

## 7. 风险与缓解

| 风险                               | 概率 | 影响 | 缓解措施                                 |
|----------------------------------|----|----|--------------------------------------|
| XML round-trip 丢失数据导致 AM 无法打开    | 高  | 严重 | 保留原始 XML + 差量合并；全面的兼容性测试矩阵           |
| 时间轴 UI 在 egui 中实现困难              | 中  | 严重 | 早期原型验证；如果 egui 不够用，考虑混合 canvas 方案    |
| bevy_alight_motion 后续改动导致编辑器需要跟进 | 高  | 中等 | flambe 仅依赖 schema 和 prelude 的稳定 API  |
| 性能问题（大型项目预览卡顿）                   | 中  | 中等 | 差量更新 ECS，避免全量重建；LOD 简化预览             |
| AM 版本差异导致兼容性问题                   | 中  | 中等 | 以 Alemon Preview7.5 4.3.4.3019 为基准测试 |

---

## 8. 结论

**可行性评估：可行，但有挑战。**

**优势**：

- bevy_alight_motion 已提供完整的解析和渲染能力，编辑器不需要重新实现底层
- 效果注册表提供了完整的效果定义元数据，可用于自动生成编辑表单
- Bevy + egui 的组合在 Rust 生态中已经有成熟的集成方案

**核心风险**：

- XML round-trip 保真是最大技术风险，需要优先攻克
- 时间轴 UI 是最大的工作量，但属于确定性可实现的工程问题

**建议**：以 Phase 1（I/O round-trip）为第一优先级，因为如果无法保证 amproj 文件的完整性，后续一切编辑功能都没有意义。
