import { Link } from 'react-router'
import { SEVERITIES, SEVERITY_LABEL } from '@/shared/types/severity'

export interface SeverityCountsProps {
  bySeverity?: Record<string, number>
  loading?: boolean
}

const numberFormat = new Intl.NumberFormat()

export function SeverityCounts({ bySeverity, loading = false }: SeverityCountsProps) {
  return (
    <ul
      aria-label="Events by severity, last 24 h"
      className="ui-panel grid grid-cols-2 overflow-hidden lg:grid-cols-4"
    >
      {SEVERITIES.map((severity) => {
        const count = bySeverity?.[severity] ?? 0
        return (
          <li
            key={severity}
            data-severity={severity}
            className="border-t-4 border-tone-solid border-l-line even:border-l lg:not-first:border-l"
          >
            <Link
              to={`/logs?severity=${severity}`}
              aria-label={`${SEVERITY_LABEL[severity]}: ${loading ? 'loading' : count} events`}
              className="block p-4 transition-colors hover:bg-hover focus-visible:-outline-offset-2"
            >
              <span className="block text-base font-semibold text-tone">
                {SEVERITY_LABEL[severity]}
              </span>
              <span
                data-testid={`count-${severity}`}
                className={`mt-1 block font-mono text-2xl font-bold lg:text-3xl tabular-nums text-fg ${loading ? 'animate-pulse opacity-40' : ''}`}
              >
                {loading ? '—' : numberFormat.format(count)}
              </span>
            </Link>
          </li>
        )
      })}
    </ul>
  )
}
