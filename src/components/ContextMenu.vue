<template>
  <Teleport to="body">
    <div v-if="visible" class="ctxmenu" ref="menuEl" :style="{ left: adjX + 'px', top: adjY + 'px' }">
      <template v-for="item in items" :key="item.action">
        <div v-if="item.separator" class="ctxmenu__sep"></div>
        <div
          v-else
          class="ctxmenu__item"
          :class="{ 'ctxmenu__item--danger': item.danger }"
          @click="$emit('select', item.action)"
        >
          {{ item.label }}
        </div>
      </template>
    </div>
    <div v-if="visible" class="ctxmenu__overlay" @click="$emit('close')" @contextmenu.prevent="$emit('close')"></div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { MenuItem } from '../composables/useContextMenu'

const props = defineProps<{
  visible: boolean; x: number; y: number; items: MenuItem[]
}>()

defineEmits<{ select: [action: string]; close: [] }>()

const menuEl = ref<HTMLElement | null>(null)
const menuW = ref(170)
const menuH = ref(200)

const adjX = computed(() => {
  const w = Math.max(menuW.value, 160)
  return (props.x + w > window.innerWidth) ? props.x - w : props.x
})
const adjY = computed(() => {
  const h = Math.max(menuH.value, 120)
  return (props.y + h > window.innerHeight) ? props.y - h : props.y
})
</script>

<style>
.ctxmenu {
  position: fixed; z-index: 9999;
  background: rgba(18,19,24,0.96);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 8px;
  padding: 4px 0;
  min-width: 160px;
  backdrop-filter: blur(12px);
}
.ctxmenu__item {
  padding: 8px 16px;
  color: rgba(255,255,255,0.8);
  font-size: 13px; cursor: pointer;
  transition: background 150ms;
}
.ctxmenu__item:hover { background: rgba(255,255,255,0.06); }
.ctxmenu__item--danger:hover { background: rgba(255,60,60,0.15); color: #ff4d4d; }
.ctxmenu__sep {
  height: 1px;
  background: rgba(255,255,255,0.06);
  margin: 4px 0;
}
.ctxmenu__overlay {
  position: fixed; inset: 0; z-index: 9998;
}
</style>
