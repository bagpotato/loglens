import { AiAnswer, useChatStream } from '@/features/ai'
import { Button } from '@/shared/ui/Button'
import { Spinner } from '@/shared/ui/Spinner'
import { Card } from './Card'

export const SUMMARY_QUESTION =
  'Summarize what happened on my servers in the last 24 hours. Start with critical and important events, group related events, and point out anything that needs action.'

export function AiSummary() {
  const { messages, isStreaming, error, send, reset } = useChatStream()
  const answer = [...messages].reverse().find((m) => m.role === 'assistant')?.content ?? ''
  const started = messages.length > 0

  const action = started ? (
    <Button variant="ghost" size="sm" onClick={reset} disabled={isStreaming}>
      Clear
    </Button>
  ) : null

  return (
    <Card title="AI summary" action={action}>
      {!started && (
        <div className="flex flex-col items-start gap-3 sm:flex-row sm:items-center sm:justify-between">
          <p className="text-base text-muted">
            Get a plain-language digest of the last 24 hours, most urgent first.
          </p>
          <Button onClick={() => send(SUMMARY_QUESTION)} className="shrink-0">
            Summarize last 24 h
          </Button>
        </div>
      )}
      {started && (
        <div aria-live="polite" aria-busy={isStreaming}>
          {answer && <AiAnswer text={answer} />}
          {isStreaming && (
            <div className="mt-3 flex items-center gap-2 text-sm text-muted">
              <Spinner label="Generating summary" />
              {answer ? 'Writing…' : 'Reading the last 24 h of logs…'}
            </div>
          )}
          {error && (
            <p role="alert" className="ui-alert mt-3">
              {error}
            </p>
          )}
        </div>
      )}
    </Card>
  )
}
