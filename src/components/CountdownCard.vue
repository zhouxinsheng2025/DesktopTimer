<template>
  <div
    class="card"
    :class="{ 'card--expired': vm.isExpired, 'card--confirm': confirming }"
    @mouseenter="hover = true"
    @mouseleave="hover = false"
    @contextmenu.prevent="$emit('ctx', $event, countdown.id)"
  >
    <div class="card__corner card__corner--tl"></div>
    <div class="card__corner card__corner--tr"></div>
    <div class="card__corner card__corner--bl"></div>
    <div class="card__corner card__corner--br"></div>

    <div
      class="card__accent"
      :style="{ background: gradientCSS, boxShadow: hover ? `0 0 8px ${glowColor}` : `0 0 4px ${glowColor}` }"
    ></div>

    <div v-if="!confirming" class="card__content">
      <div class="card__text">
        <span v-if="countdown.prefix_text">{{ countdown.prefix_text }} </span>
        <span>{{ countdown.title }} </span>
        <span v-if="countdown.suffix_text">{{ countdown.suffix_text }} </span>
      </div>
      <div
        class="card__digits"
        :style="{
          backgroundImage: gradientCSS,
          backgroundClip: 'text',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: vm.isExpired ? undefined : 'transparent',
          color: vm.isExpired ? 'rgba(255,255,255,0.3)' : undefined,
          textShadow: vm.isExpired ? 'none' : digitsGlow,
        }"
      >
        {{ vm.displayText }}
      </div>
      <div v-if="!vm.isExpired" class="card__progress">
        <div class="card__progress-track">
          <div
            class="card__progress-fill"
            :style="{ width: vm.progressPercent + '%', background: gradientCSS }"
          ></div>
          <div class="card__progress-dot" :style="{ background: glowColor }"></div>
        </div>
        <span class="card__progress-label">{{ vm.progressPercent }}%</span>
      </div>
      <div class="card__date">{{ vm.targetDateFormatted }}</div>
    </div>

    <div v-else class="card__confirm">
      <span class="card__confirm-text">确认删除？</span>
      <div class="card__confirm-actions">
        <button class="card__confirm-btn card__confirm-btn--cancel" @click="$emit('cancel-delete')">取消</button>
        <button class="card__confirm-btn card__confirm-btn--delete" @click="$emit('confirm-delete')">删除</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Countdown } from '../types'
import type { CardViewModel } from '../composables/useCountdownEngine'
import { getGradientCSS, getGlowColor } from '../types'

const props = defineProps<{
  vm: CardViewModel
  countdown: Countdown
  confirming: boolean
}>()

defineEmits<{
  ctx: [e: MouseEvent, id: string]
  'confirm-delete': []
  'cancel-delete': []
}>()

const hover = ref(false)

const gradientCSS = computed(() => getGradientCSS(props.countdown.font_style.gradient))
const glowColor = computed(() => getGlowColor(props.countdown.font_style.gradient))
const digitsGlow = computed(() => {
  if (props.vm.isExpired) return 'none'
  const blur = props.countdown.font_style.glow_intensity * 20
  return `0 0 ${blur}px ${glowColor.value}`
})
</script>
