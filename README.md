# DeskCountdown / 桌面倒计时

一个 Windows 桌面悬浮倒计时小部件，暗色极简科技感风格，支持精确到秒的多倒计时管理。

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-blue" alt="Platform">
  <img src="https://img.shields.io/badge/framework-Tauri%20v2-ffc131" alt="Tauri v2">
  <img src="https://img.shields.io/badge/frontend-Vue%203%20%2B%20TypeScript-41b883" alt="Vue 3">
  <img src="https://img.shields.io/badge/backend-Rust-000000?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/bundle-~5MB-brightgreen" alt="Bundle Size">
</p>

## 功能

- 🖥️ **桌面悬浮** — 无边框透明窗口，固定在桌面右下角，不遮挡任务栏
- ⏱️ **精确到秒** — 每 1 秒刷新，实时显示剩余天/时/分/秒
- 🎨 **渐变艺术字体** — 6 种预设渐变色 + 霓虹辉光，数字跳动带脉冲动画
- 📋 **多倒计时管理** — 右键菜单新增/编辑/复制/删除，按剩余时间自动排序
- 🔴 **过期提醒** — 归零瞬间脉冲特效，自动下沉到过期区，显示「已过期 X 天」
- 📊 **进度条** — 可视化展示已过时间占比，带流动光点装饰
- 🔧 **开机自启** — 首次启动引导，Windows 注册表写入，托盘随时开关
- 🎯 **单实例锁** — 防止重复启动，二次启动自动激活已有窗口
- ✨ **粒子背景** — 可选特效，缓慢漂浮光点营造氛围感

## 技术栈

### 前端

| 技术 | 用途 |
|------|------|
| [Vue 3](https://vuejs.org/) | 组件化 UI 框架，Composition API + `<script setup>` |
| [TypeScript](https://www.typescriptlang.org/) | 类型安全的 JavaScript 超集 |
| [Vite](https://vite.dev/) | 极速开发构建工具 |
| Vanilla CSS | 无第三方 UI 库，手写暗色主题 + HUD 科技感样式 |

核心前端模块：

- **`CountdownEngine`** (`src/composables/useCountdownEngine.ts`) — 纯前端倒计时引擎，`setInterval` 每秒从系统时钟重新计算剩余时间，无 IPC 开销，零漂移
- **`CountdownCard`** (`src/components/CountdownCard.vue`) — 卡片组件：HUD 角标、2px 主题色装饰条、CSS `background-clip: text` 渐变数字、`text-shadow` 辉光脉冲、进度条流动光点、删除确认态
- **`InlineEditor`** (`src/components/InlineEditor.vue`) — 内联编辑器：日期选择 + 快捷按钮、前后段文本双输入、3×2 渐变色块网格、辉光强度滑杆、实时预览行
- **`ContextMenu`** (`src/components/ContextMenu.vue`) — 右键弹出菜单，`Teleport` 到 body，自动翻屏定位，单实例互斥
- **`ParticlesBackground`** (`src/components/ParticlesBackground.vue`) — CSS `@keyframes` 纯动画浮动粒子，`transform` 硬件加速
- **`OnboardingToast`** (`src/components/OnboardingToast.vue`) — 首次启动自启引导卡片

> 六种渐变预设定义在 `src/types/index.ts` → `GRADIENT_PRESETS`，作为字符串键（而非原始色值）存储在数据文件中，方便全局换肤。

### 后端（Rust / Tauri v2）

| Crate | 用途 |
|-------|------|
| [tauri](https://crates.io/crates/tauri) v2 | 桌面应用框架，窗口管理、系统托盘、事件系统 |
| [tauri-plugin-dialog](https://crates.io/crates/tauri-plugin-dialog) | 原生消息框（关于对话框） |
| [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json) | JSON 序列化，结构化数据持久化 |
| [winreg](https://crates.io/crates/winreg) | Windows 注册表读写（开机自启） |
| [chrono](https://crates.io/crates/chrono) | 时间处理 |

后端模块：

- **`countdown_store`** — 数据层：`Mutex<AppData>` 保护 JSON 文件并发写入，启动时校验数据完整性，损坏自动备份恢复
- **`autostart_manager`** — `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\DeskCountdown` 注册表键写入/删除
- **`tray_manager`** — 系统托盘：5 项菜单、左键切换窗口显示/隐藏、粒子开关、`particles-toggled` 事件发射
- **`window_utils`** — 窗口定位：启动时验证坐标是否在任一屏幕工作区内，不在则回退右下角默认位置

IPC 命令（前端 → Rust）：

| 命令 | 用途 |
|------|------|
| `get_all_countdowns` | 启动时加载全部倒计时和设置 |
| `save_countdown` | 新建或更新一条倒计时 |
| `delete_countdown` | 按 ID 删除倒计时 |
| `save_window_position` | 拖拽结束后保存窗口坐标 |
| `set_autostart` / `get_autostart` | 开关/查询开机自启 |

### 单实例锁

`src-tauri/src/main.rs` 使用 Windows 原生 FFI：

```
CreateMutexW → 检测 ERROR_ALREADY_EXISTS
  → FindWindowW("桌面倒计时") → ShowWindow + SetForegroundWindow
  → 退出
```

无第三方 Windows crate 依赖，纯 `extern "system"` FFI 调用。

## 数据存储

```
%APPDATA%\countdown-timer\data.json
```

Schema：

```json
{
  "version": 1,
  "countdowns": [{
    "id": "uuid-v4",
    "title": "高考",
    "prefix_text": "距离",
    "suffix_text": "还剩",
    "created_at": "2026-07-16T12:00:00",
    "target_date": "2027-06-07T09:00:00",
    "font_style": { "gradient": "neon-cyan-blue", "glow_intensity": 0.6 },
    "sort_order": 0
  }],
  "settings": {
    "autostart": true,
    "onboarding_complete": true,
    "particles_enabled": false,
    "window_position": { "x": 100, "y": 200 }
  }
}
```

- 所有时间戳使用**本地时间**，无 UTC 时区转换
- 卸载时用户数据保留，重装后数据仍在

## 渐变色预设

| 名称 | 渐变色 | 风格 |
|------|--------|------|
| 霓虹青蓝 | `#00f0ff → #0066ff` | 科技/赛博 |
| 玫瑰金 | `#ff6b9d → #ffa751` | 温暖/优雅 |
| 火焰橙红 | `#ff6a00 → #ee0000` | 紧迫/截止 |
| 极光绿 | `#00ff88 → #00cc66` | 沉稳/进展 |
| 冰晶蓝紫 | `#7b9cff → #c084fc` | 梦幻/空灵 |
| 白金 | `#e8e8e8 → #b8944b` | 经典/高级 |

辉光强度范围 0.1–1.0，映射公式 `blur_px = intensity × 20`（2px–20px），默认 0.6 (12px)。

## 项目结构

```
deskcountdown/
├── src/                          # Vue 3 前端
│   ├── main.ts                   # 入口
│   ├── App.vue                   # 根组件，集成所有子组件 + IPC + 事件
│   ├── style.css                 # 全局样式（暗色主题、卡片、滚动条、toast）
│   ├── types/
│   │   └── index.ts              # TypeScript 类型 + 6 种渐变预设 + 工具函数
│   ├── utils/
│   │   └── format.ts             # 倒计时格式化、过期计算、进度计算
│   ├── composables/
│   │   ├── useCountdownEngine.ts # 响应式倒计时引擎（每秒刷新、排序、分组）
│   │   └── useContextMenu.ts     # 右键菜单状态管理
│   └── components/
│       ├── CountdownCard.vue     # 倒计时卡片（HUD 角标、渐变数字、进度条、辉光）
│       ├── CardList.vue          # 卡片列表（排序、空状态、过期分隔线）
│       ├── ContextMenu.vue       # 右键弹出菜单（Teleport、自动翻屏）
│       ├── InlineEditor.vue      # 内联编辑器（日历、色块、滑杆、预览）
│       ├── ParticlesBackground.vue # 粒子背景动画
│       └── OnboardingToast.vue   # 首次启动自启引导
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置（窗口、打包、NSIS）
│   ├── icons/                    # 应用图标
│   └── src/
│       ├── main.rs               # 入口 + 单实例锁（Windows FFI）
│       ├── lib.rs                # IPC 命令注册 + 应用初始化
│       ├── countdown_store.rs    # 数据模型 + JSON 持久化（Mutex）
│       ├── autostart_manager.rs  # 注册表自启管理（winreg）
│       ├── tray_manager.rs       # 系统托盘（菜单、事件、左键切换）
│       └── window_utils.rs       # 窗口定位（多屏验证）
└── docs/                         # 设计文档
    └── superpowers/
        ├── specs/                # 设计规格说明书
        └── plans/                # 实现计划
```

## 开发

### 环境要求

- [Node.js](https://nodejs.org/) ≥ 18
- [Rust](https://www.rust-lang.org/) ≥ 1.70
- Windows 10/11（WebView2 运行时，Win11 预装，Win10 由 Tauri 引导安装）

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

启动 Vite 开发服务器 + Tauri 窗口，支持热重载。

### 生产构建

```bash
npm run tauri build
```

生成：
- `src-tauri/target/release/deskcountdown.exe` — 可执行文件
- `src-tauri/target/release/bundle/nsis/DeskCountdown_*.exe` — NSIS 安装包

### 技术检查

```bash
npx vue-tsc --noEmit    # TypeScript 类型检查
cargo check              # Rust 编译检查
cargo clippy             # Rust 代码检查
```

## 安装与卸载

- **安装**：运行 NSIS 安装包，支持自定义安装路径，完成后首次启动引导自启选项
- **自启切换**：系统托盘 →「开机自启」勾选/取消
- **卸载**：通过 Windows 设置 → 应用 → DeskCountdown 卸载。卸载保留 `%APPDATA%\countdown-timer\data.json`，用户数据不丢失

## 设计文档

完整设计规格和实现计划见：

- [设计规格说明书](docs/superpowers/specs/2026-07-16-countdown-desktop-widget-design.md) — 视觉设计、架构、数据模型、交互逻辑（10 轮审核，79 个问题修正）
- [实现计划](docs/superpowers/plans/2026-07-16-countdown-implementation-plan.md) — 17 个任务、完整代码

## License

MIT
