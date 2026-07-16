export interface FontStyle {
  gradient: string
  glow_intensity: number
}

export interface Countdown {
  id: string
  title: string
  prefix_text: string
  suffix_text: string
  created_at: string
  target_date: string
  font_style: FontStyle
  sort_order: number
}

export interface WindowPosition {
  x: number
  y: number
}

export interface Settings {
  autostart: boolean
  onboarding_complete: boolean
  particles_enabled: boolean
  window_position: WindowPosition
}

export interface AppData {
  version: number
  countdowns: Countdown[]
  settings: Settings
}

export interface GradientPreset {
  key: string
  name: string
  colors: [string, string]
}

export const GRADIENT_PRESETS: GradientPreset[] = [
  { key: 'neon-cyan-blue',   name: '霓虹青蓝',  colors: ['#00f0ff', '#0066ff'] },
  { key: 'rose-gold',        name: '玫瑰金',    colors: ['#ff6b9d', '#ffa751'] },
  { key: 'flame-orange-red', name: '火焰橙红',  colors: ['#ff6a00', '#ee0000'] },
  { key: 'aurora-green',     name: '极光绿',    colors: ['#00ff88', '#00cc66'] },
  { key: 'ice-blue-purple',  name: '冰晶蓝紫',  colors: ['#7b9cff', '#c084fc'] },
  { key: 'white-gold',       name: '白金',      colors: ['#e8e8e8', '#b8944b'] },
]

export function getGradientCSS(key: string): string {
  const preset = GRADIENT_PRESETS.find(p => p.key === key)
  const [c1, c2] = preset?.colors ?? GRADIENT_PRESETS[0].colors
  return `linear-gradient(135deg, ${c1}, ${c2})`
}

export function getGlowColor(key: string): string {
  const preset = GRADIENT_PRESETS.find(p => p.key === key)
  return preset?.colors[0] ?? GRADIENT_PRESETS[0].colors[0]
}

export type EditorMode = 'create' | 'edit'
