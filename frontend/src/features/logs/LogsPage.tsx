import { useCallback, useMemo, useState } from 'react'
import type { LogEntry } from '@/shared/types/log'
import { Spinner } from '@/shared/ui/Spinner'
import { FilterBar } from './components/FilterBar'
import { LogDetail } from './components/LogDetail'
import { LogList } from './components/LogList'
import { SeverityChips } from './components/SeverityChips'
import { closestIndex, mergeEntries } from './entries'
import { useLogFilters } from './useLogFilters'
import { useLogs } from './useLogs'
import { useTail } from './useTail'

export function LogsPage() {
  const { filters, live, focus, setFilters, setLive } = useLogFilters()
  const logs = useLogs(filters)
  const tail = useTail(filters, live)
  const [selected, setSelected] = useState<LogEntry | null>(null)

  const entries = useMemo(
    () => mergeEntries(tail.entries, logs.data?.entries ?? []),
    [tail.entries, logs.data],
  )
  const focusIndex = useMemo(
    () => (focus && !logs.isPlaceholderData ? closestIndex(entries, focus) : -1),
    [entries, focus, logs.isPlaceholderData],
  )
  const onSelect = useCallback((entry: LogEntry) => setSelected(entry), [])

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="space-y-2 border-b border-line bg-surface p-3 lg:px-4">
        <FilterBar
          filters={filters}
          onChange={setFilters}
          live={live}
          onLiveChange={setLive}
          liveStatus={tail.status}
        />
        <div className="flex flex-wrap items-center justify-between gap-2">
          <SeverityChips
            selected={filters.severity}
            onChange={(severity) => setFilters({ severity })}
          />
          <span className="flex items-center gap-2 font-mono text-xs text-muted" aria-live="polite">
            {logs.isFetching ? <Spinner label="Loading logs" /> : null}
            {entries.length.toLocaleString()} entries
          </span>
        </div>
        {logs.error ? (
          <p role="alert" className="ui-alert">
            Could not load logs: {logs.error.message}
          </p>
        ) : null}
      </div>
      {logs.isPending ? (
        <div className="flex flex-1 items-center justify-center p-8">
          <Spinner label="Loading logs" />
        </div>
      ) : (
        <LogList
          entries={entries}
          selectedId={selected?.id}
          focusIndex={focusIndex}
          onSelect={onSelect}
        />
      )}
      <LogDetail entry={selected} onClose={() => setSelected(null)} />
    </div>
  )
}
