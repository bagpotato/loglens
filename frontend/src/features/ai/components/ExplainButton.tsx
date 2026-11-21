import { useState } from 'react'
import type { LogEntry } from '@/shared/types/log'
import { Button } from '@/shared/ui/Button'
import { Spinner } from '@/shared/ui/Spinner'
import { explainEntry } from '../api'

export interface ExplainButtonProps {
  entry: LogEntry
  onResult: (markdown: string) => void
}

export function ExplainButton({ entry, onResult }: ExplainButtonProps) {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const explain = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await explainEntry(entry)
      onResult(result.explanation)
    } catch (err) {
      setError(err instanceof Error && err.message ? err.message : 'Explanation failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-2">
      <Button onClick={() => void explain()} disabled={loading} className="w-full sm:w-auto">
        {loading ? <Spinner label="Explaining" /> : null}
        {loading ? 'Explaining…' : 'Explain with AI'}
      </Button>
      {error ? (
        <p role="alert" className="ui-alert">
          {error}
        </p>
      ) : null}
    </div>
  )
}
