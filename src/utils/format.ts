export function parseLocalISO(iso: string): Date {
  return new Date(iso)
}

export function formatRemaining(now: Date, target: Date): string {
  let diff = target.getTime() - now.getTime()
  if (diff <= 0) return '0分 00秒'

  const totalSeconds = Math.floor(diff / 1000)
  const days = Math.floor(totalSeconds / 86400)
  const hours = Math.floor((totalSeconds % 86400) / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60

  const pad = (n: number) => String(n).padStart(2, '0')

  if (days >= 1) {
    return `${days}天 ${pad(hours)}时 ${pad(minutes)}分 ${pad(seconds)}秒`
  }
  if (hours >= 1) {
    return `${pad(hours)}时 ${pad(minutes)}分 ${pad(seconds)}秒`
  }
  return `${pad(minutes)}分 ${pad(seconds)}秒`
}

export function formatExpired(now: Date, target: Date): string {
  const nowDay = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const targetDay = new Date(target.getFullYear(), target.getMonth(), target.getDate())
  const dayDiff = Math.floor((nowDay.getTime() - targetDay.getTime()) / 86400000)

  if (dayDiff < 1) return '今日到期'
  return `已过期 ${dayDiff} 天`
}

export function computeProgress(createdAt: string, targetDate: string, now: Date): number {
  const created = parseLocalISO(createdAt).getTime()
  const target = parseLocalISO(targetDate).getTime()
  const total = target - created
  if (total <= 0) return 0
  const elapsed = now.getTime() - created
  return Math.max(0, Math.min(100, Math.round((elapsed / total) * 100)))
}
