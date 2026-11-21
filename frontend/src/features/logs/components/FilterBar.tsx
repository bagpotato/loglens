import type { LogFilters } from '@/shared/types/filters'
import { useMeta } from '../useLogs'
import type { TailStatus } from '../useTail'
import { SearchInput } from './SearchInput'
import { TimeRangePicker } from './TimeRangePicker'

export interface FilterBarProps {
  filters: LogFilters
  onChange: (patch: Partial<LogFilters>, options?: { replace?: boolean }) => void
  live: boolean
  onLiveChange: (live: boolean) => void
  liveStatus: TailStatus
}

const selectClass = 'ui-input h-11 max-lg:text-[16px] sm:w-auto sm:min-w-36 lg:h-(--input-height)'

const LIVE_LABEL: Record<TailStatus, string> = {
  idle: 'Live',
  connecting: 'Connecting…',
  open: 'Live',
  reconnecting: 'Reconnecting…',
}

function withCurrent(values: string[] | undefined, current?: string): string[] {
  const list = values ?? []
  return current && !list.includes(current) ? [current, ...list] : list
}

export function FilterBar({ filters, onChange, live, onLiveChange, liveStatus }: FilterBarProps) {
  const meta = useMeta()
  const hosts = withCurrent(meta.data?.hosts, filters.host)
  const sources = withCurrent(meta.data?.sources, filters.source)

  return (
    <div className="flex flex-col gap-2">
      <div className="grid grid-cols-[1fr_1fr_auto] gap-2 sm:flex sm:flex-wrap sm:items-center">
        <select
          aria-label="Host"
          value={filters.host ?? ''}
          onChange={(e) => onChange({ host: e.target.value || undefined })}
          className={selectClass}
        >
          <option value="">All hosts</option>
          {hosts.map((host) => (
            <option key={host} value={host}>
              {host}
            </option>
          ))}
        </select>
        <select
          aria-label="Source"
          value={filters.source ?? ''}
          onChange={(e) => onChange({ source: e.target.value || undefined })}
          className={selectClass}
        >
          <option value="">All sources</option>
          {sources.map((source) => (
            <option key={source} value={source}>
              {source}
            </option>
          ))}
        </select>
        <div className="col-span-3 max-sm:order-last sm:min-w-64 sm:flex-1">
          <SearchInput
            value={filters.q ?? ''}
            onChange={(q) => onChange({ q: q || undefined }, { replace: true })}
          />
        </div>
        <button
          type="button"
          aria-pressed={live}
          onClick={() => onLiveChange(!live)}
          className="ui-chip h-11 justify-center rounded-md aria-pressed:border-live aria-pressed:bg-transparent aria-pressed:text-live lg:h-(--input-height)"
        >
          <span
            aria-hidden="true"
            className={`size-2 rounded-full ${live && liveStatus === 'open' ? 'animate-pulse bg-live' : 'bg-subtle'}`}
          />
          {live ? LIVE_LABEL[liveStatus] : 'Live'}
        </button>
      </div>
      <TimeRangePicker
        start={filters.start}
        end={filters.end}
        onChange={(range) => onChange({ start: range.start, end: range.end })}
      />
    </div>
  )
}
