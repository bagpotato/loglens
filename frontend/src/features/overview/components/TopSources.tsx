import { Link } from 'react-router'
import { Card, CardMessage } from './Card'

export interface TopSourcesProps {
  bySource?: Record<string, number>
  limit?: number
}

const numberFormat = new Intl.NumberFormat()

export function TopSources({ bySource, limit = 8 }: TopSourcesProps) {
  const rows = Object.entries(bySource ?? {})
    .filter(([, count]) => count > 0)
    .sort((a, b) => b[1] - a[1])
    .slice(0, limit)
  const max = rows[0]?.[1] ?? 0

  return (
    <Card title="Top sources">
      {rows.length === 0 ? (
        <CardMessage>No events in the last 24 h.</CardMessage>
      ) : (
        <ul className="-mx-2 space-y-0.5">
          {rows.map(([source, count]) => (
            <li key={source}>
              <Link
                to={`/logs?source=${encodeURIComponent(source)}`}
                className="group block rounded-md px-2 py-1.5 hover:bg-hover"
              >
                <span className="flex items-baseline justify-between gap-3 text-base">
                  <span className="truncate font-semibold text-fg">{source}</span>
                  <span className="font-mono text-sm text-muted tabular-nums">
                    {numberFormat.format(count)}
                  </span>
                </span>
                <span aria-hidden="true" className="mt-1.5 block h-1 bg-line">
                  <span
                    className="block h-full bg-fg"
                    style={{ width: `${max > 0 ? Math.max(2, (count / max) * 100) : 0}%` }}
                  />
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </Card>
  )
}
