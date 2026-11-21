import { Link } from 'react-router'
import type { LogEntry } from '@/shared/types/log'
import { Spinner } from '@/shared/ui/Spinner'
import { Card, CardMessage } from './Card'

export interface LatestCriticalProps {
  entries?: LogEntry[]
  loading?: boolean
  error?: string | null
}

const timeFormat = new Intl.DateTimeFormat(undefined, {
  month: 'short',
  day: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
})

function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  return Number.isNaN(date.getTime()) ? timestamp : timeFormat.format(date)
}

function logsLink(entry: LogEntry): string {
  const params = new URLSearchParams({
    severity: 'critical',
    host: entry.host,
    source: entry.source,
  })
  return `/logs?${params.toString()}`
}

export function LatestCritical({ entries, loading = false, error = null }: LatestCriticalProps) {
  const viewAll = (
    <Link
      to="/logs?severity=critical"
      className="text-sm font-semibold text-fg underline underline-offset-4 hover:text-muted"
    >
      View all
    </Link>
  )

  let body
  if (loading) {
    body = (
      <div className="flex justify-center py-6">
        <Spinner label="Loading critical events" />
      </div>
    )
  } else if (error) {
    body = <CardMessage>Could not load critical events: {error}</CardMessage>
  } else if (!entries || entries.length === 0) {
    body = <CardMessage>No critical events. All quiet.</CardMessage>
  } else {
    body = (
      <ol className="-mx-2 divide-y divide-line">
        {entries.map((entry) => (
          <li key={entry.id} data-severity="critical">
            <Link
              to={logsLink(entry)}
              className="block border-l-4 border-tone-solid px-3 py-2.5 hover:bg-hover focus-visible:-outline-offset-2"
            >
              <span className="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-sm text-muted">
                <time dateTime={entry.timestamp} className="font-mono text-xs tabular-nums">
                  {formatTime(entry.timestamp)}
                </time>
                <span className="font-semibold text-fg">{entry.host}</span>
                <span className="ui-badge">{entry.source}</span>
              </span>
              <span className="mt-1 line-clamp-2 font-mono text-sm break-words text-tone">
                {entry.message}
              </span>
            </Link>
          </li>
        ))}
      </ol>
    )
  }

  return (
    <Card title="Latest critical" action={viewAll}>
      {body}
    </Card>
  )
}
