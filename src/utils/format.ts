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
  const diffMs = now.getTime() - target.getTime()
  const hours = diffMs / (1000 * 60 * 60)

  if (hours < 24) return '今日到期'

  const targetDay = new Date(target.getFullYear(), target.getMonth(), target.getDate())
  const nowDay = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const days = Math.floor((nowDay.getTime() - targetDay.getTime()) / 86400000)
  return `已过期 ${days} 天`
}

export function formatTargetDate(iso: string): string {
  const d = parseLocalISO(iso)
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const h = String(d.getHours()).padStart(2, '0')
  const min = String(d.getMinutes()).padStart(2, '0')
  return `${y}-${m}-${day} ${h}:${min}`
}

export function computeProgress(createdAt: string, targetDate: string, now: Date): number {
  const created = parseLocalISO(createdAt).getTime()
  const target = parseLocalISO(targetDate).getTime()
  const total = target - created
  if (total <= 0) return 0
  const elapsed = now.getTime() - created
  return Math.max(0, Math.min(100, Math.round((elapsed / total) * 100)))
}
