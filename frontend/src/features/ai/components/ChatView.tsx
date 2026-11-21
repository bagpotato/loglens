import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { Button } from '@/shared/ui/Button'
import { useChatStream } from '../useChatStream'
import { ChatMessage } from './ChatMessage'

const SUGGESTIONS = [
  'Were there SSH brute force attempts today?',
  'Summarize the last 24 hours',
  'Which services failed or restarted recently?',
  'Any out of memory or disk errors this week?',
]

interface ChatViewProps {
  onClose?: () => void
  onNavigate?: () => void
}

export function ChatView({ onClose, onNavigate }: ChatViewProps) {
  const { messages, isStreaming, error, context, send, reset } = useChatStream()
  const [draft, setDraft] = useState('')
  const endRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    endRef.current?.scrollIntoView?.({ block: 'end' })
  }, [messages, error])

  const submit = (event?: { preventDefault: () => void }) => {
    event?.preventDefault()
    if (draft.trim() === '' || isStreaming) return
    send(draft)
    setDraft('')
  }

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && !event.shiftKey) submit(event)
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col bg-surface">
      <header className="flex items-center gap-2 border-b border-line px-4 py-2">
        <h2 className="flex-1 text-lg font-bold tracking-tight text-fg">AI assistant</h2>
        {messages.length > 0 ? (
          <Button variant="ghost" size="sm" onClick={reset}>
            New chat
          </Button>
        ) : null}
        {onClose ? (
          <Button
            variant="ghost"
            onClick={onClose}
            aria-label="Close assistant"
            className="size-10 px-0"
          >
            ✕
          </Button>
        ) : null}
      </header>

      <div className="min-h-0 flex-1 space-y-3 overflow-y-auto px-4 py-4" aria-live="polite">
        {messages.length === 0 ? (
          <div className="space-y-3">
            <p className="text-base text-muted">
              Ask about your logs in plain language. Answers cite timestamps you can open.
            </p>
            <ul className="space-y-2">
              {SUGGESTIONS.map((suggestion) => (
                <li key={suggestion}>
                  <button
                    type="button"
                    onClick={() => send(suggestion)}
                    className="min-h-11 w-full rounded-md border border-line-strong px-3 py-2 text-left text-base text-fg hover:border-ai-line hover:bg-hover"
                  >
                    {suggestion}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ) : null}
        {context ? (
          <p className="font-mono text-xs text-subtle">Analyzed {context.count} log entries</p>
        ) : null}
        {messages.map((message, index) => (
          <ChatMessage
            key={index}
            message={message}
            pending={isStreaming && index === messages.length - 1}
            onNavigate={onNavigate}
          />
        ))}
        {error ? (
          <p role="alert" className="ui-alert">
            {error}
          </p>
        ) : null}
        <div ref={endRef} />
      </div>

      <form
        onSubmit={submit}
        className="flex items-end gap-2 border-t border-line p-3 pb-[max(0.75rem,env(safe-area-inset-bottom))]"
      >
        <textarea
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={onKeyDown}
          rows={2}
          placeholder="Ask about your logs…"
          aria-label="Question"
          className="ui-input min-h-11 flex-1 resize-none max-lg:text-[16px]"
        />
        <Button type="submit" disabled={isStreaming || draft.trim() === ''}>
          Send
        </Button>
      </form>
    </div>
  )
}
