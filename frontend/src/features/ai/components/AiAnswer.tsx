import { useMemo } from 'react'
import { linkifyTimestamps } from '../linkify'
import { Markdown } from './Markdown'

export interface AiAnswerProps {
  text: string
  onNavigate?: () => void
}

/** Renders an AI answer as markdown with cited timestamps linked to the log list. */
export function AiAnswer({ text, onNavigate }: AiAnswerProps) {
  const markdown = useMemo(() => linkifyTimestamps(text), [text])
  return <Markdown onInternalLink={onNavigate}>{markdown}</Markdown>
}
