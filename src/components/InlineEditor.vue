<template>
  <div class="editor">
    <div class="editor__inner">
      <div class="editor__field">
        <label class="editor__label">标题</label>
        <input v-model="form.title" class="editor__input" maxlength="50" placeholder="输入标题" @input="validate" />
        <span v-if="titleError" class="editor__hint">标题不能为空</span>
      </div>

      <div class="editor__field">
        <label class="editor__label">日期</label>
        <div class="editor__row">
          <input v-model="form.date" class="editor__input editor__input--date" type="date" />
          <input v-model="form.time" class="editor__input editor__input--time" type="time" />
        </div>
        <div class="editor__shortcuts">
          <button class="editor__shortcut" @click="addDays(7)">一周后</button>
          <button class="editor__shortcut" @click="addDays(30)">一月后</button>
          <button class="editor__shortcut" @click="addDays(365)">一年后</button>
        </div>
      </div>

      <div class="editor__field">
        <label class="editor__label">前段</label>
        <input v-model="form.prefix" class="editor__input" maxlength="20" placeholder="例如：距离" />
      </div>
      <div class="editor__field">
        <label class="editor__label">后段</label>
        <input v-model="form.suffix" class="editor__input" maxlength="20" placeholder="例如：还剩" />
      </div>

      <div class="editor__preview">
        <span class="editor__preview-label">预览</span>
        <span class="editor__preview-text">{{ previewText }}</span>
      </div>

      <div class="editor__field">
        <label class="editor__label">字体</label>
        <div class="editor__swatches">
          <div
            v-for="p in GRADIENT_PRESETS"
            :key="p.key"
            class="editor__swatch"
            :class="{ 'editor__swatch--sel': form.gradient === p.key }"
            @click="form.gradient = p.key"
          >
            <div class="editor__swatch-bar" :style="{ background: 'linear-gradient(135deg,' + p.colors[0] + ',' + p.colors[1] + ')' }"></div>
            <span class="editor__swatch-name">{{ p.name }}</span>
          </div>
        </div>
      </div>

      <div class="editor__field">
        <label class="editor__label">辉光 {{ glowPct }}%</label>
        <input v-model="glowPct" class="editor__slider" type="range" min="0" max="100" />
      </div>

      <div class="editor__actions">
        <button class="editor__btn editor__btn--save" :disabled="!canSave" @click="$emit('save', buildData())">保存</button>
        <button class="editor__btn editor__btn--cancel" @click="$emit('cancel')">取消</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed, watch } from 'vue'
import type { Countdown, EditorMode } from '../types'
import { GRADIENT_PRESETS } from '../types'
import { formatRemaining } from '../utils/format'

const props = defineProps<{ mode: EditorMode; editData: Countdown | null }>()

defineEmits<{ save: [data: Partial<Countdown>]; cancel: [] }>()

const titleError = ref(false)
const canSave = computed(() => form.title.trim().length > 0)
const glowPct = ref(60)

const form = reactive({
  title: '',
  date: getDefaultDate(),
  time: '09:00',
  prefix: '',
  suffix: '',
  gradient: 'neon-cyan-blue',
})

function toLocalDate(d: Date): string {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

function getDefaultDate(): string {
  const d = new Date()
  d.setDate(d.getDate() + 7)
  return toLocalDate(d)
}

function validate() { titleError.value = form.title.trim().length === 0 }

function addDays(n: number) {
  const d = new Date()
  d.setDate(d.getDate() + n)
  form.date = toLocalDate(d)
}

const previewText = computed(() => {
  const p = form.prefix || ''
  const t = form.title || '___'
  const s = form.suffix || ''
  const target = new Date(form.date + 'T' + form.time)
  const now = new Date()
  const digits = target <= now ? '___天 __时 __分 __秒' : formatRemaining(now, target)
  return `${p} ${t} ${s} ${digits}`.trim()
})

function buildData(): Partial<Countdown> {
  return {
    title: form.title.trim(),
    prefix_text: form.prefix,
    suffix_text: form.suffix,
    target_date: form.date + 'T' + form.time + ':00',
    font_style: {
      gradient: form.gradient,
      glow_intensity: glowPct.value / 100,
    },
  }
}

watch(() => props.editData, (data) => {
  if (data && props.mode === 'edit') {
    form.title = data.title
    form.date = data.target_date.split('T')[0]
    form.time = data.target_date.split('T')[1].substring(0, 5)
    form.prefix = data.prefix_text
    form.suffix = data.suffix_text
    form.gradient = data.font_style.gradient
    glowPct.value = Math.round(data.font_style.glow_intensity * 100)
  } else if (props.mode === 'create') {
    form.title = data?.title || ''
    form.date = data?.target_date ? data.target_date.split('T')[0] : getDefaultDate()
    form.time = data?.target_date ? data.target_date.split('T')[1].substring(0, 5) : '09:00'
    form.prefix = data?.prefix_text || ''
    form.suffix = data?.suffix_text || ''
    form.gradient = data?.font_style.gradient || 'neon-cyan-blue'
    glowPct.value = data?.font_style.glow_intensity ? Math.round(data.font_style.glow_intensity * 100) : 60
  }
}, { immediate: true })
</script>

<style scoped>
.editor { padding: 0 8px 8px; -webkit-app-region: no-drag; }
.editor__inner {
  background: rgba(16,17,24,0.94);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 12px;
  padding: 16px;
}
.editor__field { margin-bottom: 10px; }
.editor__label { display: block; color: rgba(255,255,255,0.6); font-size: 11px; margin-bottom: 4px; }
.editor__input {
  width: 100%; padding: 6px 10px;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 6px; color: rgba(255,255,255,0.85); font-size: 13px;
  box-sizing: border-box; outline: none;
}
.editor__input:focus { border-color: rgba(255,255,255,0.25); }
.editor__row { display: flex; gap: 8px; }
.editor__input--date { flex: 2; }
.editor__input--time { flex: 1; }
.editor__shortcuts { display: flex; gap: 6px; margin-top: 6px; }
.editor__shortcut {
  padding: 3px 10px; background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.08); border-radius: 4px;
  color: rgba(255,255,255,0.5); font-size: 11px; cursor: pointer;
  transition: all 150ms;
}
.editor__shortcut:hover { background: rgba(255,255,255,0.1); color: rgba(255,255,255,0.75); }
.editor__hint { color: #ff6b6b; font-size: 11px; margin-top: 3px; display: block; }

.editor__preview {
  padding: 10px 12px; background: rgba(0,0,0,0.3);
  border-radius: 8px; margin-bottom: 10px;
}
.editor__preview-label { color: rgba(255,255,255,0.35); font-size: 10px; display: block; margin-bottom: 4px; }
.editor__preview-text { color: rgba(255,255,255,0.8); font-size: 14px; font-weight: 600; }

.editor__swatches { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
.editor__swatch {
  padding: 8px; border: 1px solid rgba(255,255,255,0.08);
  border-radius: 8px; cursor: pointer; text-align: center;
  transition: border-color 150ms;
}
.editor__swatch--sel { border-color: rgba(255,255,255,0.4); }
.editor__swatch-bar { height: 8px; border-radius: 4px; margin-bottom: 4px; }
.editor__swatch-name { color: rgba(255,255,255,0.6); font-size: 10px; }

.editor__slider { width: 100%; accent-color: #00b4ff; }

.editor__actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 14px; }
.editor__btn { padding: 7px 24px; border: none; border-radius: 6px; font-size: 13px; cursor: pointer; transition: opacity 150ms; }
.editor__btn:hover { opacity: 0.85; }
.editor__btn--save { background: rgba(0,180,255,0.2); color: #00b4ff; }
.editor__btn--save:disabled { opacity: 0.35; cursor: not-allowed; }
.editor__btn--cancel { background: rgba(255,255,255,0.06); color: rgba(255,255,255,0.5); }
</style>
