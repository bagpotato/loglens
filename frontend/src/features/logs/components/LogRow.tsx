import { memo } from 'react'
import type { LogEntry } from '@/shared/types/log'
import { Badge } from '@/shared/ui/Badge'
import { formatTimestamp } from '../entries'

export interface LogRowProps {
  entry: LogEntry
  selected: boolean
  highlighted: boolean
  onSelect: (entry: LogEntry) => void
}

export const LogRow = memo(function LogRow({
  entry,
  selected,
  highlighted,
  onSelect,
}: LogRowProps) {
  const isAi = entry.severity_source === 'ai'

  return (
    <button
      type="button"
      onClick={() => onSelect(entry)}
      title={isAi && entry.reason ? `AI: ${entry.reason}` : undefined}
      data-severity={entry.severity}
      data-selected={selected || undefined}
      data-highlighted={highlighted || undefined}
      className="ui-row flex-col lg:flex-row lg:items-center lg:gap-4"
    >
      <span className="flex min-w-0 shrink-0 items-center gap-2 text-xs text-muted">
        <span className="flex items-center gap-1 lg:w-28">
          <Badge severity={entry.severity} />
          {isAi ? (
            <span
              aria-label="Severity classified by AI"
              className="rounded-sm border border-ai-line bg-ai-bg px-1 text-2xs leading-4 font-bold text-ai"
            >
              AI
            </span>
          ) : null}
        </span>
        <time dateTime={entry.timestamp} className="font-mono whitespace-nowrap lg:w-40">
          {formatTimestamp(entry.timestamp)}
        </time>
        <span className="max-w-40 truncate font-semibold text-fg lg:w-28">{entry.host}</span>
        <span className="truncate text-subtle lg:w-16">{entry.source}</span>
      </span>
      <span className="block min-w-0 flex-1 truncate font-mono text-sm text-fg">
        {entry.message}
      </span>
    </button>
  )
})
