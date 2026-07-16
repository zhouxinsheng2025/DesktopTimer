# Windows Desktop Countdown Widget — Design Spec

**Date**: 2026-07-16  
**Status**: Approved  
**Topic**: 桌面悬浮倒计时小部件

---

## 1. Overview

A desktop countdown widget for Windows that floats on the desktop, displays multiple countdowns sorted by remaining time, supports customizable gradient text and second-level precision, and optionally starts with the system.

**App Name**: DeskCountdown / 桌面倒计时

**Tech Stack**: Tauri v2 (Rust backend + WebView2 frontend) + Vue 3 + vanilla CSS

**Package size**: ~5 MB | **Runtime memory**: ~30 MB

---

## 2. Visual Design

### 2.1 Overall Style

Dark minimal + HUD sci-fi aesthetic. The window is a frameless, transparent-background Tauri window positioned at the desktop layer (not always-on-top by default). The WebView2 HTML root (`html, body`) must also set `background: transparent` to allow the desktop to show through the window — otherwise WebView2 renders a default white background that blocks the transparency effect.

**Default position**: On first launch (no saved `window_position`), the window appears at the bottom-right corner of the primary monitor's working area: 40px from the right edge, 40px from the bottom edge. (Using the working area — which automatically excludes the taskbar — correctly handles taskbars positioned at any screen edge.) On subsequent launches, the saved position is validated — if the saved coordinates fall outside any connected screen's working area (e.g., because an external monitor was disconnected), the window falls back to the default position.

**Window dragging**: Since the window has no title bar (`decorations: false`), the user can drag it by clicking and holding any empty area of the window — including the margin space around the card list area, the empty state placeholder, and the gaps between cards. In Tauri, this is implemented by setting the CSS `-webkit-app-region: drag` on the root container and `-webkit-app-region: no-drag` on interactive elements (cards, buttons, inputs) so clicks on those elements are not consumed by the drag handler. The window saves its position to `settings.window_position` on drag-end (debounced to 500ms). Additionally, on application quit (tray → "退出"), the current position is saved synchronously to cover the edge case where the user drags and immediately quits before the debounce fires.

**Window close behavior**: The window has no title bar and thus no close button. Pressing Alt+F4 hides the window to the system tray rather than quitting — consistent with the tray's "显示/隐藏" action. To fully quit, the user uses the tray menu → "退出".

### 2.2 Card Design

Each countdown card:

| Element | Specification |
|---------|---------------|
| Background | `rgba(10, 11, 16, 0.92)` with subtle top-to-bottom gradient (slightly brighter at top). Noise texture overlay at 3% opacity. |
| Border | 1px `rgba(255, 255, 255, 0.06)`, 12px border-radius |
| Corner accents | Four L-shaped 45° HUD markers (`┌┐└┘`) at each corner, 3-4× brighter than the border, the only bright border elements |
| Left accent bar | 2px vertical bar on the left edge, colored with the card's gradient theme color, with a soft glow |
| Card spacing | 8px vertical gap between cards |
| Card width | Fixed at 300px. Card internal padding: 16px horizontal, 14px vertical. Window padding: 8px transparent margin on all sides around the card list, so cards don't touch the window edge. |
| Text color | Prefix, title, and suffix: `rgba(255, 255, 255, 0.85)`. Target date label: `rgba(255, 255, 255, 0.45)`, smaller font size (11px). Countdown digits: gradient-colored (see §2.3). |
| Text overflow | Title: single-line, `text-overflow: ellipsis`, max 50 characters. Prefix/suffix: max 20 characters each. Target date label: always fits on one line. |
| Hover effect | Border brightness transitions from 6% → 25% over 200ms; left accent bar glow doubles |

### 2.3 Typography & Gradient Presets

Six preset gradient styles for countdown digits:

| Preset Name | Gradient Colors | Use Case Vibe |
|-------------|----------------|---------------|
| 霓虹青蓝 (Neon Cyan-Blue) | `#00f0ff → #0066ff` | Tech/cyber |
| 玫瑰金 (Rose Gold) | `#ff6b9d → #ffa751` | Warm/elegant |
| 火焰橙红 (Flame Orange-Red) | `#ff6a00 → #ee0000` | Urgent/deadline |
| 极光绿 (Aurora Green) | `#00ff88 → #00cc66` | Calm/progress |
| 冰晶蓝紫 (Ice Blue-Purple) | `#7b9cff → #c084fc` | Dreamy/ethereal |
| 白金 (White Gold) | `#e8e8e8 → #b8944b` | Classic/premium |

Each digit character carries a `text-shadow` glow in the same color family. On each second-tick, the glow radius briefly expands by ~30% then shrinks back over 300ms (CSS transition on the shadow).

**Countdown digit display format**:
- Days ≥ 1: `D天 HH时 MM分 SS秒` (e.g., `326天 12时 45分 30秒`)
- Days = 0, hours ≥ 1: `HH时 MM分 SS秒` (omit days)
- Days = 0, hours = 0: `MM分 SS秒` (omit days and hours — this is the minimal format, always showing minutes and seconds)
- All units are zero-padded to 2 digits except days (no padding). Units separated by a single space.

**Customizable glow intensity**: 0.1–1.0, controlling the `text-shadow` blur radius via `blur_px = intensity × 20` (range: 2px–20px). Default for new countdowns: 0.6 (12px blur). The default gradient preset for new countdowns is 霓虹青蓝 (first in the list).

### 2.4 Progress Bar

- 3px height, full card width minus padding
- Left segment: card's gradient color; right segment: `rgba(255,255,255,0.08)`
- A subtle light dot travels left-to-right every ~3 seconds
- Percentage label on the right, small font
- **Progress calculation**: The progress bar represents the percentage of total time elapsed between `created_at` (when the countdown was first created) and `target_date`. Formula: `elapsed / (elapsed + remaining) × 100%`. For a newly created countdown, the bar starts near 0% and fills to 100% as the deadline approaches. If `elapsed + remaining === 0` (created_at equals target_date, an edge case), the bar shows 0%. If the user edits a countdown and changes the `target_date`, the `created_at` timestamp is reset to the edit time so the progress bar restarts from 0%. If only non-date fields (title, text, font) are changed, `created_at` is left unchanged — the progress bar is not affected.

### 2.5 Background Particles

- 8–12 floating light dots across the entire window background
- Opacity 10–15%, slow random vertical drift
- Purely cosmetic. Toggleable via the system tray menu → "特效 ☑". Disabled by default on first launch to prioritize performance. State is stored in `settings.particles_enabled`.

### 2.6 Right-Click Context Menu

Dark semi-transparent background, 8px border-radius. Each item hover: a 2px theme-colored vertical bar appears on the left edge. Delete option gets a red hover accent.

**Positioning**: The menu appears at the cursor position. If the menu would overflow the right or bottom edge of the screen, it flips to the left/above the cursor instead. Only one context menu instance exists at a time — opening a new menu (or right-clicking elsewhere) closes the previous one.

Menu items (when right-clicking a card): 新增倒计时 / 编辑此倒计时 / 复制此倒计时 / ─ / 删除此倒计时

Menu items (when right-clicking empty area between cards or window margin): only 新增倒计时.

**Copy behavior**: Selecting "复制此倒计时" opens the inline editor in create mode with all fields pre-filled from the source card. The title automatically gets a "副本" suffix (e.g., "高考 副本"). No card is inserted into the list yet — the user can adjust any field. On "保存", a new UUID is generated, the card is inserted into the list and persisted to disk. On "取消", nothing is persisted — no phantom card left behind.

### 2.7 Delete Confirmation

To avoid accidental deletion without breaking immersion with a modal dialog:

1. User clicks "删除此倒计时" in the context menu.
2. The target card transitions into a **confirmation state**: the card content fades out and is replaced by "确认删除？" text, flanked by two buttons — "删除" (red accent) and "取消" (dimmed). The card's height is pinned to its original value during this transition to prevent window jitter.
3. The card background gains a subtle red tint (`rgba(255, 0, 0, 0.08)` overlay) during the 200ms transition.
4. Clicking "取消", clicking outside the card, or pressing `Escape` restores the original content.
5. Clicking "删除" triggers the actual deletion: the card collapses vertically over 200ms, the list slides to fill the gap, the window shrinks to match (per §2.10), and the data is removed from storage.

### 2.8 Empty State

When no countdowns exist (first launch or all deleted):

- The card list area shows a dotted border placeholder rectangle (2px dashed, `rgba(255,255,255,0.10)`, 12px border-radius).
- Inside: a subtle `+` icon (opacity 15%) centered above the hint text "右键新建倒计时" (opacity 30%, small font).
- Right-clicking anywhere on the empty state area opens the context menu with only "新增倒计时" available.
- When the user selects "新增倒计时", the empty state fades out and the inline editor expands in its place. If the user cancels without saving (and no cards exist), the empty state fades back in.
- This ensures the user immediately understands the app is functional and knows how to get started.

### 2.9 Expired Countdowns

When a countdown's `target_date` has passed:

- **Zero-point transition**: At the exact second the countdown reaches zero: (1) the card instantly re-sorts to the bottom of the list (expired group), (2) the card emits a brief "pulse flash" in its new position — glow intensity surges to `min(glow_intensity × 2, 1.0)` for 200ms, then fades to zero over the next 300ms, then transitions into the expired visual state. The re-sort happens before the animation so the card pulses in its final location. This only fires for countdowns that were actively counting down (target_date was in the future when created). Cards created with a past date skip the pulse and render directly in expired state.
- **Visual**: After the pulse flash (or immediately, if created already-expired), the entire card desaturates (grayscale filter 60%) and opacity drops to 40%. The gradient digits and glow are suppressed — text renders in flat `rgba(255,255,255,0.3)`.
- **Text**: The countdown digits are replaced by "今日到期" if the deadline was less than 24 hours ago, or "已过期 X 天" if ≥ 1 day (X = full calendar days since the deadline date, increments at midnight local time).
- **Sort position**: Expired countdowns sink to the **bottom** of the list, below all active countdowns (inverting the normal sort rule). Among expired cards, more recently expired cards appear above older ones.
- **Progress bar**: Hidden — no meaningful progress to display.
- **Actions**: The user can right-click to edit (change the date to a future date, which re-activates it) or delete.
- **Separation**: A subtle 1px separator line with "已过期" label divides the two groups **only when both groups are non-empty** (see §4.4). If all cards are active, no separator appears. If all cards are expired, no separator appears.

### 2.10 Window Height Auto-Adaptation

The Tauri window height dynamically matches the total height of all cards (plus the editor panel, if open):

- **Card added**: Window smoothly expands downward by the card height + gap, using a 250ms ease-out transition.
- **Card deleted**: Window smoothly contracts upward over 250ms, sliding the remaining cards into place.
- **Editor toggled**: Same smooth expand/contract when the inline editor panel opens or closes.
- **No scrollbar**: The window always exactly fits the content. Scrollbar only appears as a fallback if the total height exceeds the screen's working area.
- **Minimum size**: Window has a minimum height (empty state height, ~120px) and a minimum width of 348px (300px card + 32px card padding + 16px window padding).
- **Position clamping**: If the window is near the bottom of the screen and expands, it shifts upward to stay fully visible on screen.
- **Position validation on startup**: On launch, the saved `window_position` is checked against all currently connected screens. If the saved coordinates fall entirely outside every screen's working area (e.g., a disconnected external monitor), the window reverts to the default position (bottom-right of primary screen, per §2.1).

This is implemented by having the frontend measure content height after DOM updates and call `appWindow.setSize()` directly via Tauri's frontend window API (`@tauri-apps/api/window`). No IPC round-trip is needed — the frontend has direct access to window resize. The frontend recalculates after every card add/delete/edit toggle. **Implementation note**: Vue DOM updates are async — the frontend must `await nextTick()` after the reactive state change before measuring the new content height, otherwise the measurement will read stale (pre-update) dimensions.

---

## 3. Architecture

```
┌─────────────────────────────────────────────┐
│           Desktop Floating Window            │
│  ┌───────────────────────────────────────┐  │
│  │         Card List Container (Vue)      │  │
│  │  ┌─────────────────────────────────┐  │  │
│  │  │  Inline Editor Panel (toggle)    │  │  │
│  │  └─────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────┐  │  │
│  │  │  Countdown Card 1 (farthest)     │  │  │
│  │  ├─────────────────────────────────┤  │  │
│  │  │  Countdown Card 2                │  │  │
│  │  ├─────────────────────────────────┤  │  │
│  │  │  Countdown Card N (closest)      │  │  │
│  │  └─────────────────────────────────┘  │  │
│  └───────────────────────────────────────┘  │
│                  ▲                          │
│                  │ IPC (invoke)             │
│                  ▼                          │
│  ┌───────────────────────────────────────┐  │
│  │           Rust Backend (Tauri)         │  │
│  │  ┌──────────┐  ┌───────────────────┐  │  │
│  │  │ Countdown │  │ Autostart Manager  │  │  │
│  │  │ Store     │  │ (HKCU Registry)   │  │  │
│  │  ├──────────┤  ├───────────────────┤  │  │
│  │  │ JSON R/W │  │ System Tray       │  │  │
│  │  │ Engine   │  │ Menu Manager      │  │  │
│  │  └──────────┘  └───────────────────┘  │  │
│  │  ┌──────────────────────────────────┐  │  │
│  │  │  Window Utils (frameless,        │  │  │
│  │  │  desktop-layer, opacity)         │  │  │
│  │  └──────────────────────────────────┘  │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 3.1 Frontend Components (Vue 3)

| Component | Responsibility |
|-----------|---------------|
| `App.vue` | Root layout, initializes countdown engine, loads data from backend on mount |
| `CountdownCard.vue` | Single card: prefix text + title + suffix text + gradient digits, progress bar, corner accents, glow animation, hover effect |
| `CardList.vue` | Vertical list container, sorts cards by `target_date` (closest at bottom), handles scroll if needed |
| `ContextMenu.vue` | Right-click popup menu, emits events for each action, positioned at cursor |
| `InlineEditor.vue` | Expandable edit panel at top of card list: title input, calendar date picker with quick shortcuts, prefix/suffix text inputs with live preview, gradient swatch card selector (3×2 grid), glow intensity slider |
| `CountdownEngine.ts` | Pure JS module: a `setInterval(1000)` loop that computes remaining time for each card and exposes reactive state. Runs entirely in the frontend — no IPC per tick. The interval is cleared in the `onUnmounted` lifecycle hook to prevent memory leaks on hot-reload or component teardown. |
| `ParticlesBackground.vue` | Renders 8–12 floating light dots behind the card list. Pure CSS animation, toggled via `settings.particles_enabled`. |

### 3.2 Backend Modules (Rust/Tauri)

| Module | Responsibility |
|--------|---------------|
| `countdown_store` | Read/write `data.json` in `%APPDATA%\countdown-timer\`. CRUD operations exposed via `#[tauri::command]`. Validates data integrity on load. All write operations are serialized through a `std::sync::Mutex` to prevent data loss from concurrent IPC calls. |
| `autostart_manager` | Writes/deletes registry key `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\CountdownTimer` pointing to the installed executable. Exposed via `set_autostart(bool)` command. |
| `tray_manager` | Creates system tray icon with menu: Show/Hide, Particles toggle, Autostart toggle, About, Quit. Emits `particles-toggled` event to frontend. |
| `window_utils` | Configures the Tauri window: `decorations: false`, `transparent: true`, `always_on_top: false`, `skip_taskbar: true`. Saves/restores window position. Validates saved position against available screens on startup. (Dynamic window resize is handled on the frontend via `@tauri-apps/api/window` — see §2.10.) |

### 3.3 IPC Commands

| Command | Direction | Purpose |
|---------|-----------|---------|
| `get_all_countdowns` | Frontend → Rust | Load all countdowns and settings on startup (returns both arrays from data.json) |
| `save_countdown(countdown)` | Frontend → Rust | Create or update a countdown |
| `delete_countdown(id)` | Frontend → Rust | Remove a countdown |
| `set_autostart(enabled)` | Frontend → Rust | Toggle registry autostart |
| `get_autostart()` | Frontend → Rust | Read current autostart state (used by onboarding flow to confirm registry state after user choice) |
| `get_app_version()` | Frontend → Rust | Get app version for about dialog |
| `save_window_position(x, y)` | Frontend → Rust | Persist window coordinates after drag-end |
| `set_particles_enabled(bool)` | Frontend → Rust | Reserved for future frontend-initiated toggle — unused in v1 (tray menu handles this on the Rust side) |

### 3.4 Data Flow

```
Startup:
  1. Tauri main process starts → creates transparent window → loads Vue app
  2. Vue app invokes get_all_countdowns() → Rust reads data.json → returns JSON
  3. CountdownEngine initializes reactive state → starts 1-second timer
  4. Cards render sorted by target_date descending (farthest at top, closest at bottom)

User adds/edits a countdown:
  1. Right-click empty area → "新增倒计时", or right-click card → "编辑此倒计时" → InlineEditor expands
  2. User fills form, clicks "保存"
  3. Frontend invokes save_countdown(data) → Rust writes data.json (serialized via Mutex)
  4. Frontend refreshes card list, CountdownEngine picks up the new entry

Per-second update:
  1. CountdownEngine.setInterval fires every 1000ms
  2. Iterates all cards, recomputes remaining time, updates reactive state
  3. Vue reactivity triggers DOM updates for changed digits only
  4. Glow animation fires on second-change via CSS class toggle

Copy countdown:
  1. Right-click card → "复制此倒计时" → InlineEditor expands in create mode with cloned data
  2. User adjusts fields, clicks "保存"
  3. Same as "add" flow from step 3 onwards (new UUID → save → refresh)

Delete countdown:
  1. Right-click card → "删除此倒计时" → card enters confirmation state
  2. User clicks "删除" → frontend invokes delete_countdown(id) → Rust writes data.json
  3. Card collapses over 200ms, list slides to fill the gap, window shrinks to match

Right-click:
  1. User right-clicks a card → ContextMenu appears at cursor position
  2. Menu option selected → corresponding action fires
  3. Menu closes on click-outside or Escape

Tray-initiated change (particles toggle):
  1. User toggles "特效" in tray menu → Rust handler updates settings.particles_enabled in store
  2. Rust emits `particles-toggled` event with new state → frontend listener updates reactive flag
  3. ParticlesBackground.vue mounts/unmounts via v-if, no page reload needed
```

---

## 4. Data Model

### 4.1 Storage Location

`%APPDATA%\countdown-timer\data.json` (resolves to `C:\Users\<user>\AppData\Roaming\countdown-timer\data.json`)

All timestamps (`created_at`, `target_date`) are stored in **local time** without UTC offset. The countdown engine computes remaining time using the system's current local clock. No timezone conversion is performed — a countdown set to 09:00 in Beijing will fire at 09:00 Beijing time regardless of where the machine physically is.

### 4.2 Schema

```json
{
  "version": 1,
  "countdowns": [
    {
      "id": "uuid-v4",
      "title": "高考",
      "prefix_text": "距离",
      "suffix_text": "还剩",
      "created_at": "2026-07-16T12:00:00",
      "target_date": "2027-06-07T09:00:00",
      "font_style": {
        "gradient": "neon-cyan-blue",
        "glow_intensity": 0.6
      },
      "sort_order": 0
    }
  ],
  "settings": {
    "autostart": true,
    "onboarding_complete": true,
    "particles_enabled": false,
    "window_position": { "x": 100, "y": 200 }
  }
}
```

### 4.3 Field Notes

- `prefix_text`: Text displayed before the title. Example: `"距离"` renders as "距离 **高考** 还剩 326天 12时 45分 30秒".
- `suffix_text`: Text displayed after the title, before the countdown digits. Example: `"还剩"`.
- The card renders as: **prefix_text** + **title** + **suffix_text** + **digits**. This avoids the confusing `__` placeholder approach — users simply fill in two natural-language fields.
- `sort_order`: Reserved for future manual drag-to-reorder. Currently cards are sorted by `target_date` descending (farthest deadline at the top, closest at the bottom). If two cards share the same target date, the tiebreaker is `title` alphabetical (locale-aware).
- `font_style.gradient`: Key into the 6-preset gradient map. Not raw color values — this keeps the data file clean and allows preset palette updates without migrating data.
- `font_style.glow_intensity`: Float 0.1–1.0, maps to `text-shadow` blur radius via formula `blur_px = intensity × 20` (range: 2px–20px). The default intensity for new countdowns is 0.6 (→ 12px blur).
- `id`: UUID v4 string, generated by the frontend via `crypto.randomUUID()` (natively supported in WebView2) at the moment the user clicks "保存" in the inline editor. The backend treats it as an opaque identifier — no generation logic needed in Rust.
- `created_at`: ISO 8601 timestamp in **local time** (no UTC offset), set by the frontend when the countdown is first saved. Used as the progress bar's start anchor (see §2.4). Reset to the current time if the user later changes the `target_date` via edit.
- `onboarding_complete`: Set to `true` after the first-launch autostart toast is dismissed or acted upon. Prevents the toast from showing again.
- `particles_enabled`: Toggles background particle effects. Default `false` on first launch. Toggled via tray menu; the Rust backend emits a `particles-toggled` event so the frontend can react without polling.

### 4.4 Sort Order

Cards are sorted in two groups:

1. **Active countdowns** (target_date in the future): sorted by `target_date` **descending**, rendered top-to-bottom. The countdown with the farthest deadline (largest date) appears at the **top**, and the nearest deadline (smallest date) appears at the **bottom** — matching the user's preference of "closer to deadline → closer to the bottom of the screen." If two cards share the same `target_date`, the tiebreaker is `title` alphabetical (locale-aware, `Intl.Collator`).
2. **Expired countdowns** (target_date in the past): placed below all active countdowns, sorted by `target_date` descending (most recently expired at the top of the expired group). Same-date tiebreaker: `title` alphabetical.

A subtle separator with "已过期" label divides the two groups **only when both groups are non-empty**. If all cards are active, no separator appears. If all cards are expired, no separator appears — the first expired card sits at the top of the list without a divider.

---

## 5. Inline Editor Panel

### 5.1 Layout

```
┌─  ─────────────────────────────────────  ─┐
│                                            │
│  标题: [  高考          ]                  │
│                                            │
│  日期: [  2027-06-07   ]  [  09:00  ]     │
│        [一周后] [一月后] [一年后]          │  ← 快捷按钮
│                                            │
│  前段: [  距离    ]                        │
│  后段: [  还剩    ]                        │
│  ┌─ 预览 ──────────────────────────┐      │
│  │  距离 高考 还剩  326天 12时 45分 30秒 │  │
│  └─────────────────────────────────┘      │
│                                            │
│  字体: [■霓虹蓝紫] [■玫瑰金] [■火焰橙红]  │  ← 色块卡片选择
│        [■极光绿]   [■冰晶蓝紫] [■白金]    │
│  辉光: [▓▓▓▓▓▓▓░░░] 70%                   │
│                                            │
│  [保存]  [取消]                            │
│                                            │
└─  ─────────────────────────────────────  ─┘
```

### 5.2 Behavior

- Appears at the top of the card list when adding (no card selected) or editing (right-clicked card). The editor operates in two modes tracked by an internal `mode` state:
  - **`create` mode**: Triggered by right-click empty area → "新增倒计时", or right-click card → "复制此倒计时". For new countdowns: fields are empty except the date, which defaults to today + 7 days (the "一周后" shortcut). For copied countdowns: all fields are pre-filled from the source card. On save, a new UUID is generated and the countdown is appended to the list.
  - **`edit` mode**: Triggered by right-click card → "编辑此倒计时". All fields are pre-filled from the existing card. On save, the existing entry is updated in place by matching `id`.
- **Editor conflict guard**: If the editor is already open and the user right-clicks anywhere (card or empty area), the context menu is suppressed — a small toast appears immediately near the cursor: "请先保存或取消当前编辑" (fades out after 2 seconds). This prevents the user from navigating a menu whose options will all be rejected.
- **Click-outside behavior**: Clicking outside the editor (on a card or empty area) does nothing — the editor stays open. The user must explicitly click "保存" or "取消" to dismiss it. This prevents accidental loss of in-progress edits.
- Card list pushes down with a 200ms smooth expand/collapse transition.

**Custom text (prefix + suffix)**:
- Two simple text inputs: "前段" (prefix, before the title) and "后段" (suffix, after the title but before the countdown digits).
- A live preview row instantly shows the full rendered text: `前段 + 标题 + 后段 + [倒计时数字]`. The countdown digits use the same format as the card display (§2.3). As a defensive fallback (e.g., during component initialization before the default date is applied), digits show as `___天 __时 __分 __秒`.
- No placeholder syntax to learn — users just type natural text in the right fields.

**Date picker**:
- Clicking the date field opens a mini calendar panel (not a native OS picker).
- Today's date is highlighted. Selected date gets a glow ring.
- Three quick-access shortcut buttons below the date field: "一周后" (+7 days), "一月后" (+30 days), "一年后" (+365 days). Each click advances the date from today — not from the currently selected date.
- Time is a separate inline time picker (HH:MM), defaulting to 09:00.

**Font selection**:
- Six gradient presets displayed as small swatch cards in a 3×2 grid.
- Each swatch shows a thumbnail preview of the gradient colors + the preset name.
- Clicking a swatch selects it with a highlighted border; clicking a different one deselects the previous.
- Glow intensity is a range slider. In **edit mode**, dragging updates the target card's text-shadow in real time. In **create mode**, there is no card to preview — the slider value is reflected in the preview row's digit styling only. The slider displays as a percentage (0%–100%) for UX clarity; the stored value is the decimal equivalent (0.0–1.0, e.g., 70% → `glow_intensity: 0.7`).

**Validation**:
- Title is required. If the title input is empty or whitespace-only, the "保存" button is disabled (grayed out) with a subtle hint "标题不能为空" below the input.
- Prefix and suffix are optional — either or both can be empty.
- Target date must be a valid date; the date picker prevents invalid selections by design.

- "保存" invokes `save_countdown`, collapses the panel, and refreshes the card list.
- "取消" discards changes and collapses.
- **Keyboard**: Pressing `Escape` while the editor is open is equivalent to clicking "取消" (discard changes, collapse editor). Pressing `Enter` while the editor is open and the save button is enabled is equivalent to clicking "保存". The `Escape` key also closes the context menu (§2.6) and cancels the delete confirmation state (§2.7, step 4).

---

## 6. Installation & Autostart

### 6.1 Installer

- Built with Tauri bundler → produces `.exe` NSIS installer with `installMode: "both"` to support custom install paths (`.msi` is also supported but NSIS provides better custom-install-path UX).
- Installer includes a **custom install path** page where the user chooses the target folder.
- **Uninstall**: The NSIS uninstaller removes the application binaries and the registry autostart key. User data (`%APPDATA%\countdown-timer\data.json`) is intentionally left behind so countdowns are not lost on reinstall.

### 6.2 First-Launch Onboarding

Instead of an installer checkbox (which Tauri's NSIS installer doesn't natively support for custom logic), autostart is handled by the app on first launch:

1. On first launch, `data.json` doesn't exist. The backend creates it with default settings: `autostart: false`, `onboarding_complete: false`. (Note: §4.2 shows a post-onboarding snapshot where `onboarding_complete` is `true` — that is not the initial default.)
2. The frontend detects first launch (empty countdown list + no prior run marker) and shows a small onboarding toast above the empty state: **"开机时自动启动？"** with buttons "开启" (primary, highlighted) and "暂不" (dimmed).
3. If the user clicks "开启", the frontend invokes `set_autostart(true)` and the setting is saved.
4. If the user clicks "暂不" or dismisses, autostart stays off. The user can enable it later via the system tray menu.
5. A marker flag `onboarding_complete: true` is written to `settings` in `data.json` so the toast never appears again.

### 6.3 Autostart Mechanism

```rust
// HKCU\Software\Microsoft\Windows\CurrentVersion\Run\CountdownTimer
// Value: "C:\Program Files\CountdownTimer\countdown-timer.exe"
```

- Toggled anytime via system tray → "开机自启" (checked/unchecked).
- The Rust `autostart_manager` module handles registry write/delete using the `winreg` crate.
- On app startup, the backend reads the current registry state and reports it to the frontend.

---

## 7. System Tray

The application ships with a `.ico` icon file (bundled via Tauri's `tauri.conf.json` → `bundle.icon`), used for both the system tray and the installer. The tray icon should be a simple, recognizable glyph — a minimalist hourglass or countdown-clock silhouette — at 16×16, 32×32, 48×48, and 256×256 resolutions.

**Left-click**: Toggles window visibility (show/hide). Same behavior as the tray menu's "显示/隐藏" item.

**Right-click**: Opens the tray context menu:

| Menu Item | Action |
|-----------|--------|
| 显示/隐藏 | Toggle window visibility. The menu label reflects current state: shows "隐藏" when window is visible, "显示" when hidden. Since the window has `skip_taskbar: true` (no taskbar button), the tray is the **only** way to restore a hidden window — this is noted in the UI via the tray icon's tooltip ("桌面倒计时 - 左键切换显示"). |
| 特效 ☑ | Toggle background particle effects (shows checkmark when enabled). Disabled by default on first launch. |
| 开机自启 ☑ | Toggle autostart (shows checkmark when enabled) |
| 关于 | Show app name and version via a native Rust message box (Tauri v2: `tauri_plugin_dialog::MessageDialog`) |
| 退出 | Quit the application entirely. On next launch, the window reappears at its last saved position. |

---

## 8. Error Handling

| Scenario | Handling |
|----------|----------|
| `data.json` missing (first run) | Rust creates default file with empty countdowns array and default settings |
| `data.json` corrupted | Back up as `data.json.bak`, reinitialize with defaults, show tray notification |
| `data.json` write failure (permissions / disk full) | Log error, show tray notification "无法保存数据", no crash, app continues with in-memory state |
| Invalid date (past date) | Accepted — countdown rendered in expired state per §2.9 |
| Registry write fails (autostart) | Log error, show tray notification "无法设置开机自启", no crash |
| WebView2 not installed | Tauri's WebView2 bootstrapper handles this during install; if it fails, show a native error dialog with a manual download link |
| Second instance launched | A Windows OS-level named mutex (`DeskCountdown/SingleInstance`) created via `CreateMutexW` in the Rust main function prevents multiple instances. The Tauri window is assigned a unique class name (`DeskCountdown/MainWindow`) at creation. A second launch detects the existing mutex, finds the first instance's HWND via `FindWindowW` with the class name, calls `SetForegroundWindow` + `ShowWindow(SW_RESTORE)` to activate it, then exits. |

---

## 9. Performance Constraints

- 1-second interval runs entirely in JS, no IPC per tick → minimal overhead.
- Gradient text uses CSS `background-clip: text` + `background-image: linear-gradient(...)`, all GPU-accelerated.
- Particle animation uses CSS `transform` only (no layout recalc), limited to 12 particles.
- Window is `skip_taskbar: true` to avoid taskbar clutter.
- Target frame rate: 60fps on integrated graphics.
- **DPI scaling**: All CSS dimensions use relative units (`rem`, `%`, `px` at the root based on device pixel ratio). WebView2 automatically handles high-DPI rendering on Windows; no special configuration is needed beyond ensuring the Tauri window has `"useHdpi": true` (default in Tauri v2). The window position stored in `settings` uses physical pixels, which are validated against logical screen coordinates corrected for DPI scale on each launch.

---

## 10. Future Considerations (Not in v1)

| Feature | Reason Deferred |
|---------|----------------|
| Drag-to-reorder cards | Sorting is automatic by deadline; manual ordering is a nice-to-have |
| Multiple gradient presets on one card (e.g., days vs hours different colors) | Adds complexity to the editor without clear user demand |
| Cloud sync | Explicitly scoped out — local-only for v1 |
| Count-up mode (elapsed time since an event) | Different use case, can evaluate |
| Pomodoro / timer mode | Entirely different interaction model |
| Sound notifications when countdown reaches zero | Future enhancement |
| Global hotkey to show/hide the window | Adds convenience for keyboard-centric users |

---

## 11. Design Decisions Log

| Decision | Rationale |
|----------|-----------|
| Tauri over Electron | 24× smaller bundle, 5× less RAM, better for a desktop widget that runs 24/7 |
| Vue 3 over vanilla JS | Lightweight reactivity saves manual DOM management for the per-second updates |
| Closest deadline at bottom | User preference — puts urgent items closer to taskbar/attention zone |
| Inline editor over modal dialog | Avoids window management complexity, keeps everything in one place |
| Inline editor at top of list | User chose right-click menu, but editing inline at the top is better UX than a separate window |
| Gradient presets as string keys | Allows updating the palette globally without touching user data |
| JSON file storage | Simplest persistent storage; no need for SQLite for a handful of countdowns |
| No mouse-passthrough by default | Desktop-icon-level interaction, not a wallpaper overlay |
| Particle background | Subtle enough to add atmosphere without being distracting; easily toggled off |
