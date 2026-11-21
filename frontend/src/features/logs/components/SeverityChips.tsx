import { SEVERITIES, SEVERITY_LABEL, type Severity } from '@/shared/types/severity'

export interface SeverityChipsProps {
  selected: Severity[]
  onChange: (next: Severity[]) => void
}

export function SeverityChips({ selected, onChange }: SeverityChipsProps) {
  const toggle = (severity: Severity) => {
    const next = selected.includes(severity)
      ? selected.filter((s) => s !== severity)
      : SEVERITIES.filter((s) => s === severity || selected.includes(s))
    onChange(next)
  }

  return (
    <div
      role="group"
      aria-label="Severity"
      className="flex items-center gap-1.5 overflow-x-auto [scrollbar-width:none] sm:flex-wrap"
    >
      {SEVERITIES.map((severity) => {
        const active = selected.includes(severity)
        return (
          <button
            key={severity}
            type="button"
            aria-pressed={active}
            data-severity={severity}
            onClick={() => toggle(severity)}
            className="ui-chip"
          >
            {SEVERITY_LABEL[severity]}
          </button>
        )
      })}
      {selected.length === 0 ? (
        <span className="text-xs whitespace-nowrap text-subtle">All severities</span>
      ) : null}
    </div>
  )
}
