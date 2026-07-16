<template>
  <div class="app-root" @contextmenu.prevent="onRootCtx">
    <ParticlesBackground :enabled="particlesEnabled" />

    <OnboardingToast
      :visible="showOnboarding"
      @enable="onOnboardEnable"
      @dismiss="onOnboardDismiss"
    />

    <InlineEditor
      v-if="editorVisible"
      :mode="editorMode"
      :edit-data="editingCard"
      @save="onEditorSave"
      @cancel="onEditorCancel"
    />

    <CardList
      :active-cards="activeCards"
      :expired-cards="expiredCards"
      :separator-visible="separatorVisible"
      :confirming-id="confirmingId"
      :countdowns="countdowns"
      @card-ctx="onCardCtx"
      @confirm-del="onConfirmDel"
      @cancel-del="confirmingId = null"
      @empty-ctx="onRootCtx"
    />

    <ContextMenu
      :visible="menuVisible"
      :x="menuX" :y="menuY" :items="menuItems"
      @select="onMenuSelect"
      @close="closeMenu"
    />

    <div v-if="toastVisible" class="toast">{{ toastMsg }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { nextTick } from 'vue'

import type { Countdown, EditorMode } from './types'
import { useCountdownEngine } from './composables/useCountdownEngine'
import { useContextMenu } from './composables/useContextMenu'
import type { MenuItem } from './composables/useContextMenu'

import CardList from './components/CardList.vue'
import ContextMenu from './components/ContextMenu.vue'
import InlineEditor from './components/InlineEditor.vue'
import ParticlesBackground from './components/ParticlesBackground.vue'
import OnboardingToast from './components/OnboardingToast.vue'

// ── Engine ──
const { countdowns, activeCards, expiredCards, separatorVisible, loadFromBackend } = useCountdownEngine()

// ── Context menu ──
const { menuVisible, menuX, menuY, menuItems, openMenu, closeMenu } = useContextMenu()

// ── Editor ──
const editorVisible = ref(false)
const editorMode = ref<EditorMode>('create')
const editingCard = ref<Countdown | null>(null)

// ── Delete confirm ──
const confirmingId = ref<string | null>(null)

// ── Onboarding ──
const showOnboarding = ref(false)
const particlesEnabled = ref(false)

// ── Toast ──
const toastMsg = ref('')
const toastVisible = ref(false)
let toastTimer: ReturnType<typeof setTimeout> | null = null
function showToast(msg: string) {
  toastMsg.value = msg
  toastVisible.value = true
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastVisible.value = false }, 2000)
}

// ── Menu items ──
const CARD_MENU: MenuItem[] = [
  { label: '新增倒计时', action: 'add' },
  { label: '编辑此倒计时', action: 'edit' },
  { label: '复制此倒计时', action: 'copy' },
  { label: '', action: 'sep', separator: true },
  { label: '删除此倒计时', action: 'delete', danger: true },
]
const EMPTY_MENU: MenuItem[] = [{ label: '新增倒计时', action: 'add' }]

let menuTargetId: string | null = null

function onCardCtx(e: MouseEvent, id: string) {
  menuTargetId = id
  if (editorVisible.value) { showToast('请先保存或取消当前编辑'); return }
  openMenu(e.clientX, e.clientY, CARD_MENU)
}

function onRootCtx(e: MouseEvent) {
  menuTargetId = null
  if (editorVisible.value) { showToast('请先保存或取消当前编辑'); return }
  openMenu(e.clientX, e.clientY, EMPTY_MENU)
}

function onMenuSelect(action: string) {
  closeMenu()
  switch (action) {
    case 'add': openEditor('create', null); break
    case 'edit': {
      const card = countdowns.value.find(c => c.id === menuTargetId)
      if (card) openEditor('edit', card)
      break
    }
    case 'copy': {
      const card = countdowns.value.find(c => c.id === menuTargetId)
      if (card) {
        const copy: Countdown = { ...structuredClone(card), id: '', title: card.title + ' 副本' }
        openEditor('create', copy)
      }
      break
    }
    case 'delete': confirmingId.value = menuTargetId; break
  }
}

// ── Editor ──
function openEditor(mode: EditorMode, data: Countdown | null) {
  editorMode.value = mode
  editingCard.value = data
  editorVisible.value = true
  resizeWindow()
}

async function onEditorSave(data: Partial<Countdown>) {
  const now = new Date().toISOString().replace('Z', '')
  if (editorMode.value === 'create') {
    const id = crypto.randomUUID()
    const card: Countdown = {
      id, title: data.title!, prefix_text: data.prefix_text || '', suffix_text: data.suffix_text || '',
      created_at: now, target_date: data.target_date!, font_style: data.font_style!, sort_order: 0,
    }
    await invoke('save_countdown', { countdown: card })
  } else if (editorMode.value === 'edit' && editingCard.value) {
    const prev = editingCard.value
    const updated: Countdown = {
      ...prev, title: data.title!, prefix_text: data.prefix_text || '', suffix_text: data.suffix_text || '',
      target_date: data.target_date!, font_style: data.font_style!,
      created_at: data.target_date !== prev.target_date ? now : prev.created_at,
    }
    await invoke('save_countdown', { countdown: updated })
  }
  editorVisible.value = false
  editingCard.value = null
  await loadFromBackend()
  resizeWindow()
}

function onEditorCancel() {
  editorVisible.value = false
  editingCard.value = null
  resizeWindow()
}

// ── Delete ──
async function onConfirmDel(id: string) {
  await invoke('delete_countdown', { id })
  confirmingId.value = null
  await loadFromBackend()
  resizeWindow()
}

// ── Onboarding ──
async function onOnboardEnable() {
  await invoke('set_autostart', { enabled: true })
  await invoke('complete_onboarding')
  showOnboarding.value = false
}
async function onOnboardDismiss() {
  await invoke('complete_onboarding')
  showOnboarding.value = false
}

// ── Window resize ──
async function resizeWindow() {
  await nextTick()
  const el = document.querySelector('.app-root') as HTMLElement
  if (!el) return
  try {
    await getCurrentWindow().setSize(new PhysicalSize(348, el.scrollHeight))
  } catch (_) {}
}

// ── Events ──
const unlistenParticles = listen<boolean>('particles-toggled', (e) => { particlesEnabled.value = e.payload })

// ── Keyboard ──
function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (menuVisible.value) closeMenu()
    else if (confirmingId.value) confirmingId.value = null
    else if (editorVisible.value) onEditorCancel()
  }
}
document.addEventListener('keydown', onKeyDown)
onUnmounted(async () => {
  document.removeEventListener('keydown', onKeyDown)
  ;(await unlistenParticles)()
})

// ── Init ──
onMounted(async () => {
  const data = await loadFromBackend() as any
  if (data?.settings) {
    if (!data.settings.onboarding_complete) showOnboarding.value = true
    particlesEnabled.value = data.settings.particles_enabled ?? false
  }
  resizeWindow()
})
</script>

<style scoped>
.app-root { width: 348px; min-height: 120px; -webkit-app-region: drag; }
</style>

<style>
.toast {
  position: fixed; top: 8px; left: 50%; transform: translateX(-50%); z-index: 10000;
  padding: 6px 16px; background: rgba(255,255,255,0.1); border-radius: 6px;
  color: rgba(255,255,255,0.75); font-size: 12px; pointer-events: none;
  animation: toastIn 200ms ease-out;
}
@keyframes toastIn {
  from { opacity: 0; transform: translateX(-50%) translateY(-10px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
