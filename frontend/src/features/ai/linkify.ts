const TIMESTAMP_RE =
  /\[(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2}))\](?!\()/g
const CODE_RE = /(```[\s\S]*?(?:```|$)|`[^`\n]*`)/
const WINDOW_MS = 2 * 60 * 1000
const ALL_SEVERITIES = 'critical,important,warning,normal'

/** Link to the logs view around `timestamp` (+-2 minutes, all severities, focused on it). */
export function logLinkFor(timestamp: string): string | null {
  const ms = Date.parse(timestamp)
  if (Number.isNaN(ms)) return null
  const start = new Date(ms - WINDOW_MS).toISOString()
  const end = new Date(ms + WINDOW_MS).toISOString()
  return (
    `/logs?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}` +
    `&severity=${ALL_SEVERITIES}&focus=${encodeURIComponent(timestamp)}`
  )
}

function linkifyText(text: string): string {
  return text.replace(TIMESTAMP_RE, (match, timestamp: string) => {
    const href = logLinkFor(timestamp)
    return href ? `[${timestamp}](${href})` : match
  })
}

/** Turns `[RFC3339]` citations into markdown links, leaving code spans and blocks untouched. */
export function linkifyTimestamps(markdown: string): string {
  return markdown
    .split(CODE_RE)
    .map((part, index) => (index % 2 === 0 ? linkifyText(part) : part))
    .join('')
}
