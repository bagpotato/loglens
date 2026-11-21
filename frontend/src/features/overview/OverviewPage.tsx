import { ApiError } from '@/shared/api/client'
import { AiSummary } from './components/AiSummary'
import { LatestCritical } from './components/LatestCritical'
import { SeverityCounts } from './components/SeverityCounts'
import { TopSources } from './components/TopSources'
import { useLatestCritical, useStats } from './api'

function message(error: unknown): string | null {
  if (!error) return null
  return error instanceof ApiError || error instanceof Error ? error.message : 'Unknown error'
}

export function OverviewPage() {
  const stats = useStats()
  const critical = useLatestCritical()
  const statsError = message(stats.error)

  return (
    <div className="mx-auto w-full max-w-6xl space-y-4 px-4 py-5 sm:px-6 lg:space-y-5 lg:py-7">
      <header>
        <h1 className="text-2xl font-bold tracking-tight text-fg">Overview</h1>
        <p className="text-base text-muted">Last 24 hours across all hosts</p>
      </header>
      {statsError && (
        <p role="alert" className="ui-alert">
          Could not load statistics: {statsError}
        </p>
      )}
      <SeverityCounts bySeverity={stats.data?.by_severity} loading={stats.isPending} />
      <AiSummary />
      <div className="grid gap-4 lg:grid-cols-3 lg:gap-5">
        <div className="lg:col-span-2">
          <LatestCritical
            entries={critical.data}
            loading={critical.isPending}
            error={message(critical.error)}
          />
        </div>
        <TopSources bySource={stats.data?.by_source} />
      </div>
    </div>
  )
}
