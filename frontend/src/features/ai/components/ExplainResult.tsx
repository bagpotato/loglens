import { Markdown } from './Markdown'

export interface ExplainResultProps {
  markdown: string
}

export function ExplainResult({ markdown }: ExplainResultProps) {
  return (
    <section
      aria-label="AI explanation"
      className="rounded-md border border-ai-line border-l-2 border-l-ai bg-ai-bg p-3"
    >
      <p className="text-sm font-bold text-ai">AI explanation</p>
      <Markdown>{markdown}</Markdown>
    </section>
  )
}
