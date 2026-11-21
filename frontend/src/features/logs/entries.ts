import type { LogEntry } from '@/shared/types/log'

export const MAX_ENTRIES = 10_000

/** Prepends entries received oldest-first so the result stays newest-first, capped. */
export function prependEntries(
  current: LogEntry[],
  incoming: LogEntry[],
  cap = MAX_ENTRIES,
): LogEntry[] {
  const next = [...incoming].reverse().concat(current)
  return next.length > cap ? next.slice(0, cap) : next
}

/** Live entries first, then queried entries, without duplicates. */
export function mergeEntries(live: LogEntry[], queried: LogEntry[], cap = MAX_ENTRIES): LogEntry[] {
  if (live.length === 0) return queried.length > cap ? queried.slice(0, cap) : queried
  const seen = new Set<string>()
  const result: LogEntry[] = []
  for (const entry of live.concat(queried)) {
    if (seen.has(entry.id)) continue
    seen.add(entry.id)
    result.push(entry)
    if (result.length >= cap) break
  }
  return result
}

/** Index of the entry whose timestamp is closest to `timestamp`, or -1. */
export function closestIndex(entries: LogEntry[], timestamp: string): number {
  const target = Date.parse(timestamp)
  if (Number.isNaN(target)) return -1
  let best = -1
  let bestDistance = Infinity
  entries.forEach((entry, index) => {
    const distance = Math.abs(Date.parse(entry.timestamp) - target)
    if (distance < bestDistance) {
      best = index
      bestDistance = distance
    }
  })
  return best
}

export function isLogEntry(value: unknown): value is LogEntry {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  return (
    typeof v.id === 'string' &&
    typeof v.timestamp === 'string' &&
    typeof v.message === 'string' &&
    typeof v.severity === 'string'
  )
}

export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp
  const pad = (n: number) => String(n).padStart(2, '0')
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ` +
    `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
  )
}
