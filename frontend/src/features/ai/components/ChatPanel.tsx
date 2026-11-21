import type { KeyboardEvent } from 'react'
import { useIsDesktop } from '@/shared/hooks/useMediaQuery'
import { ChatView } from './ChatView'

export interface ChatPanelProps {
  onClose?: () => void
}

/** Fills its container on desktop (side panel slot); full-screen overlay on mobile. */
export function ChatPanel({ onClose }: ChatPanelProps) {
  const isDesktop = useIsDesktop()

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === 'Escape' && onClose) {
      event.stopPropagation()
      onClose()
    }
  }

  return (
    <div
      onKeyDown={onKeyDown}
      className={
        isDesktop ? 'flex h-full min-h-0 flex-1 flex-col' : 'fixed inset-0 z-50 flex flex-col'
      }
    >
      <ChatView onClose={onClose} onNavigate={isDesktop ? undefined : onClose} />
    </div>
  )
}
