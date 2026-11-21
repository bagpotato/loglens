import { Spinner } from '@/shared/ui/Spinner'
import type { ChatTurn } from '../types'
import { AiAnswer } from './AiAnswer'

export interface ChatMessageProps {
  message: ChatTurn
  pending?: boolean
  onNavigate?: () => void
}

export function ChatMessage({ message, pending = false, onNavigate }: ChatMessageProps) {
  if (message.role === 'user') {
    return (
      <div className="flex justify-end">
        <p className="max-w-[85%] rounded-md bg-accent px-3 py-2 text-base whitespace-pre-wrap text-on-accent">
          {message.content}
        </p>
      </div>
    )
  }

  return (
    <div className="max-w-full rounded-md border border-line border-l-2 border-l-ai bg-raised px-3 py-1">
      {message.content === '' && pending ? (
        <div className="flex items-center gap-2 py-2 text-base text-muted">
          <Spinner label="Thinking" />
          Searching logs…
        </div>
      ) : (
        <AiAnswer text={message.content} onNavigate={onNavigate} />
      )}
    </div>
  )
}
