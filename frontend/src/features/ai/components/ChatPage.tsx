import { ChatView } from './ChatView'

/** Route wrapper: the chat fills the whole content area of the layout. */
export function ChatPage() {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <ChatView />
    </div>
  )
}
