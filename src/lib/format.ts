export function clampNumber(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}

export function formatTimecode(valueMs: number) {
  const totalSeconds = Math.max(0, Math.floor(valueMs / 1000))
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60

  if (hours > 0) {
    return [hours, minutes, seconds]
      .map((part, index) => (index === 0 ? String(part) : String(part).padStart(2, '0')))
      .join(':')
  }

  return [minutes, seconds].map((part) => String(part).padStart(2, '0')).join(':')
}

export function formatPercent(value: number) {
  return `${value.toFixed(value >= 100 ? 0 : 1)}%`
}

export function formatStatusLabel(status: string) {
  return status.slice(0, 1).toUpperCase() + status.slice(1)
}

export function sanitizeFilenameStem(value: string) {
  return (
    value
      .replace(/[<>:"/\\|?*]+/g, '')
      .replace(/\s+/g, '_')
      .slice(0, 80) || 'watermelon_clip'
  )
}
