import { useCallback, useEffect, useRef, useState } from 'react'
import { streamChat } from './api'
import type { ChatContext, ChatEvent, ChatStream, ChatTurn } from './types'

function errorText(error: unknown): string {
  return error instanceof Error && error.message ? error.message : 'The AI request failed'
}

function appendToAssistant(messages: ChatTurn[], text: string): ChatTurn[] {
  const last = messages[messages.length - 1]
  if (!last || last.role !== 'assistant') return [...messages, { role: 'assistant', content: text }]
  return [...messages.slice(0, -1), { role: 'assistant', content: last.content + text }]
}

export function useChatStream(): ChatStream {
  const [messages, setMessages] = useState<ChatTurn[]>([])
  const [isStreaming, setIsStreaming] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [context, setContext] = useState<ChatContext | null>(null)
  const controllerRef = useRef<AbortController | null>(null)

  useEffect(() => () => controllerRef.current?.abort(), [])

  const send = useCallback(
    (question: string) => {
      const text = question.trim()
      if (text === '' || controllerRef.current) return
      const history = messages.filter((m) => m.content.trim() !== '')
      const controller = new AbortController()
      controllerRef.current = controller
      setMessages([...history, { role: 'user', content: text }, { role: 'assistant', content: '' }])
      setError(null)
      setContext(null)
      setIsStreaming(true)

      const onEvent = (event: ChatEvent) => {
        if (controller.signal.aborted) return
        switch (event.type) {
          case 'filters':
            setContext({ filters: event.filters, count: event.count })
            break
          case 'delta':
            setMessages((prev) => appendToAssistant(prev, event.text))
            break
          case 'error':
            setError(event.message)
            break
          case 'done':
            break
        }
      }

      streamChat(text, history, controller.signal, onEvent)
        .catch((err: unknown) => {
          if (!controller.signal.aborted) setError(errorText(err))
        })
        .finally(() => {
          if (controllerRef.current !== controller) return
          controllerRef.current = null
          setIsStreaming(false)
        })
    },
    [messages],
  )

  const reset = useCallback(() => {
    controllerRef.current?.abort()
    controllerRef.current = null
    setMessages([])
    setIsStreaming(false)
    setError(null)
    setContext(null)
  }, [])

  return { messages, isStreaming, error, context, send, reset }
}
