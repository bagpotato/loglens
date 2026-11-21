import { useRef, useState, type ReactNode } from 'react'

export function CodeBlock({ children }: { children?: ReactNode }) {
  const preRef = useRef<HTMLPreElement>(null)
  const [copied, setCopied] = useState(false)

  const copy = async () => {
    const text = preRef.current?.textContent ?? ''
    try {
      await navigator.clipboard.writeText(text.replace(/\n$/, ''))
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      setCopied(false)
    }
  }

  return (
    <div className="relative my-2">
      <pre
        ref={preRef}
        className="ui-code overflow-x-auto p-3 pr-16 leading-relaxed text-fg [&_code]:bg-transparent [&_code]:p-0"
      >
        {children}
      </pre>
      <button
        type="button"
        onClick={() => void copy()}
        className="ui-btn ui-btn--secondary ui-btn--sm absolute top-2 right-2 h-7 text-xs"
      >
        {copied ? 'Copied' : 'Copy'}
      </button>
    </div>
  )
}
