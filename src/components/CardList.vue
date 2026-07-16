<template>
  <div class="card-list">
    <div v-if="activeCards.length === 0 && expiredCards.length === 0" class="card-list__empty" @contextmenu.prevent="$emit('empty-ctx', $event)">
      <div class="card-list__empty-icon">+</div>
      <div class="card-list__empty-text">右键新建倒计时</div>
    </div>

    <template v-else>
      <CountdownCard
        v-for="vm in activeCards"
        :key="vm.id"
        :vm="vm"
        :countdown="getRaw(vm.id)"
        :confirming="confirmingId === vm.id"
        @ctx="(e, id) => $emit('card-ctx', e, id)"
        @confirm-delete="$emit('confirm-del', vm.id)"
        @cancel-delete="$emit('cancel-del')"
      />

      <div v-if="separatorVisible" class="card-list__sep">
        <span>已过期</span>
      </div>

      <CountdownCard
        v-for="vm in expiredCards"
        :key="vm.id"
        :vm="vm"
        :countdown="getRaw(vm.id)"
        :confirming="confirmingId === vm.id"
        @ctx="(e, id) => $emit('card-ctx', e, id)"
        @confirm-delete="$emit('confirm-del', vm.id)"
        @cancel-delete="$emit('cancel-del')"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import type { CardViewModel } from '../composables/useCountdownEngine'
import type { Countdown } from '../types'
import CountdownCard from './CountdownCard.vue'

const props = defineProps<{
  activeCards: CardViewModel[]
  expiredCards: CardViewModel[]
  separatorVisible: boolean
  confirmingId: string | null
  countdowns: Countdown[]
}>()

defineEmits<{
  'card-ctx': [e: MouseEvent, id: string]
  'confirm-del': [id: string]
  'cancel-del': []
  'empty-ctx': [e: MouseEvent]
}>()

function getRaw(id: string): Countdown {
  return props.countdowns.find(c => c.id === id)!
}
</script>

<style scoped>
.card-list { padding: 8px; -webkit-app-region: drag; min-height: 120px; }

.card-list__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 104px;
  border: 2px dashed rgba(255,255,255,0.10);
  border-radius: 12px;
  -webkit-app-region: no-drag;
}
.card-list__empty-icon {
  font-size: 32px;
  color: rgba(255,255,255,0.15);
  line-height: 1;
  margin-bottom: 6px;
}
.card-list__empty-text {
  color: rgba(255,255,255,0.30);
  font-size: 12px;
}

.card-list__sep {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  margin: 4px 0;
  -webkit-app-region: no-drag;
}
.card-list__sep::before, .card-list__sep::after {
  content: '';
  flex: 1; height: 1px;
  background: rgba(255,255,255,0.06);
}
.card-list__sep span {
  margin: 0 12px;
  color: rgba(255,255,255,0.25);
  font-size: 11px;
}
</style>
