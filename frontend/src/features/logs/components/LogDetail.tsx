import { useState, type ReactNode } from 'react'
import { ExplainButton, ExplainResult } from '@/features/ai'
import type { LogEntry } from '@/shared/types/log'
import { Badge } from '@/shared/ui/Badge'
import { Drawer } from '@/shared/ui/Drawer'
import { formatTimestamp } from '../entries'

export interface LogDetailProps {
  entry: LogEntry | null
  onClose: () => void
}

const SOURCE_LABEL: Record<LogEntry['severity_source'], string> = {
  rule: 'Rule',
  ai: 'AI',
  default: 'Default',
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="grid grid-cols-[6.5rem_1fr] gap-2 py-2">
      <dt className="text-muted">{label}</dt>
      <dd className="min-w-0 break-words text-fg">{children}</dd>
    </div>
  )
}

function LogDetailBody({ entry }: { entry: LogEntry }) {
  const [explanation, setExplanation] = useState<string | null>(null)
  const isAi = entry.severity_source === 'ai'

  return (
    <div className="space-y-4 text-base" data-severity={entry.severity}>
      <pre className="ui-code overflow-x-auto border-l-4 border-l-tone-solid p-3 leading-relaxed break-words whitespace-pre-wrap text-fg">
        {entry.message}
      </pre>
      <ExplainButton entry={entry} onResult={setExplanation} />
      {explanation !== null ? <ExplainResult markdown={explanation} /> : null}
      {isAi && entry.reason ? (
        <p className="rounded-md border border-ai-line bg-ai-bg px-3 py-2 text-ai">
          <span className="font-bold">AI classification: </span>
          {entry.reason}
        </p>
      ) : null}
      <dl className="divide-y divide-line">
        <Field label="Severity">
          <span className="inline-flex items-center gap-2">
            <Badge severity={entry.severity} />
            <span className="text-xs text-muted">by {SOURCE_LABEL[entry.severity_source]}</span>
          </span>
        </Field>
        <Field label="Time">
          <time dateTime={entry.timestamp}>{formatTimestamp(entry.timestamp)}</time>
          <span className="block font-mono text-xs text-subtle">{entry.timestamp}</span>
        </Field>
        <Field label="Host">{entry.host}</Field>
        <Field label="Source">{entry.source}</Field>
        {entry.unit ? <Field label="Unit">{entry.unit}</Field> : null}
        {entry.container ? <Field label="Container">{entry.container}</Field> : null}
        <Field label="ID">
          <span className="font-mono text-xs">{entry.id}</span>
        </Field>
      </dl>
    </div>
  )
}

export function LogDetail({ entry, onClose }: LogDetailProps) {
  return (
    <Drawer open={entry !== null} onClose={onClose} title="Log entry">
      {entry ? <LogDetailBody key={entry.id} entry={entry} /> : null}
    </Drawer>
  )
}
