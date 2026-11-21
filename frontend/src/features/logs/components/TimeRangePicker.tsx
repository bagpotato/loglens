import { useState } from 'react'
import { formatTimestamp } from '../entries'

const PRESETS = [
  { label: '15m', ms: 15 * 60_000 },
  { label: '1h', ms: 60 * 60_000 },
  { label: '24h', ms: 24 * 60 * 60_000 },
  { label: '7d', ms: 7 * 24 * 60 * 60_000 },
] as const

export interface TimeRange {
  start?: string
  end?: string
}

export interface TimeRangePickerProps extends TimeRange {
  onChange: (range: TimeRange) => void
}

function toLocalInput(iso?: string): string {
  return iso ? formatTimestamp(iso).replace(' ', 'T').slice(0, 16) : ''
}

function fromLocalInput(value: string): string | undefined {
  if (!value) return undefined
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? undefined : date.toISOString()
}

function describe({ start, end }: TimeRange): string {
  if (!start && !end) return 'Last 24 h'
  const from = start ? formatTimestamp(start) : '…'
  return end ? `${from} → ${formatTimestamp(end)}` : `Since ${from}`
}

const inputClass = 'ui-input h-11 w-auto max-lg:text-[16px] lg:h-(--input-height)'

export function TimeRangePicker({ start, end, onChange }: TimeRangePickerProps) {
  const [custom, setCustom] = useState(false)

  return (
    <div className="flex items-center gap-2 overflow-x-auto [scrollbar-width:none] sm:flex-wrap">
      <div role="group" aria-label="Time range" className="flex gap-1.5">
        {PRESETS.map((preset) => (
          <button
            key={preset.label}
            type="button"
            onClick={() => {
              setCustom(false)
              onChange({ start: new Date(Date.now() - preset.ms).toISOString(), end: undefined })
            }}
            className="ui-chip rounded-md"
          >
            {preset.label}
          </button>
        ))}
        <button
          type="button"
          aria-expanded={custom}
          onClick={() => setCustom((c) => !c)}
          className="ui-chip rounded-md"
        >
          Custom
        </button>
      </div>
      {custom ? (
        <div className="flex flex-wrap items-center gap-2">
          <input
            type="datetime-local"
            aria-label="From"
            value={toLocalInput(start)}
            onChange={(e) => onChange({ start: fromLocalInput(e.target.value), end })}
            className={inputClass}
          />
          <input
            type="datetime-local"
            aria-label="To"
            value={toLocalInput(end)}
            onChange={(e) => onChange({ start, end: fromLocalInput(e.target.value) })}
            className={inputClass}
          />
        </div>
      ) : (
        <span className="font-mono text-xs whitespace-nowrap text-muted">
          {describe({ start, end })}
        </span>
      )}
    </div>
  )
}
