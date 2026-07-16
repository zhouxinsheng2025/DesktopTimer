import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { Countdown } from '../types'
import { formatRemaining, formatExpired, computeProgress, parseLocalISO } from '../utils/format'
import { invoke } from '@tauri-apps/api/core'

export interface CardViewModel {
  id: string
  title: string
  prefixText: string
  suffixText: string
  displayText: string
  progressPercent: number
  targetDateFormatted: string
  isExpired: boolean
  glowIntensity: number
}

export function useCountdownEngine() {
  const countdowns = ref<Countdown[]>([])
  const now = ref(new Date())

  let intervalId: ReturnType<typeof setInterval> | null = null

  const cardViewModels = computed<CardViewModel[]>(() => {
    return countdowns.value.map(c => {
      const target = parseLocalISO(c.target_date)
      const isExpired = now.value >= target
      const displayText = isExpired
        ? formatExpired(now.value, target)
        : formatRemaining(now.value, target)
      const progressPercent = isExpired
        ? 0
        : computeProgress(c.created_at, c.target_date, now.value)

      return {
        id: c.id,
        title: c.title,
        prefixText: c.prefix_text,
        suffixText: c.suffix_text,
        displayText,
        progressPercent,
        targetDateFormatted: `${target.getFullYear()}-${String(target.getMonth() + 1).padStart(2, '0')}-${String(target.getDate()).padStart(2, '0')}`,
        isExpired,
        glowIntensity: c.font_style.glow_intensity,
      }
    })
  })

  const activeCards = computed(() =>
    cardViewModels.value
      .filter(c => !c.isExpired)
      .sort((a, b) => {
        const aC = countdowns.value.find(x => x.id === a.id)!
        const bC = countdowns.value.find(x => x.id === b.id)!
        const dateDiff = new Date(bC.target_date).getTime() - new Date(aC.target_date).getTime()
        if (dateDiff !== 0) return dateDiff
        return a.title.localeCompare(b.title, 'zh-CN')
      })
  )

  const expiredCards = computed(() =>
    cardViewModels.value
      .filter(c => c.isExpired)
      .sort((a, b) => {
        const aC = countdowns.value.find(x => x.id === a.id)!
        const bC = countdowns.value.find(x => x.id === b.id)!
        const dateDiff = new Date(bC.target_date).getTime() - new Date(aC.target_date).getTime()
        if (dateDiff !== 0) return dateDiff
        return a.title.localeCompare(b.title, 'zh-CN')
      })
  )

  const separatorVisible = computed(() =>
    activeCards.value.length > 0 && expiredCards.value.length > 0
  )

  async function loadFromBackend() {
    const data = await invoke<any>('get_all_countdowns')
    countdowns.value = data.countdowns
    return data
  }

  function startTimer() {
    now.value = new Date()
    intervalId = setInterval(() => {
      now.value = new Date()
    }, 1000)
  }

  onMounted(() => {
    startTimer()
  })

  onUnmounted(() => {
    if (intervalId !== null) {
      clearInterval(intervalId)
    }
  })

  return {
    countdowns,
    activeCards,
    expiredCards,
    separatorVisible,
    cardViewModels,
    loadFromBackend,
  }
}
