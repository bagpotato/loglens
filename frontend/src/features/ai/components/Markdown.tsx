import { useMemo } from 'react'
import ReactMarkdown, { type Components } from 'react-markdown'
import { Link } from 'react-router'
import { CodeBlock } from './CodeBlock'

// Images in AI output could load attacker URLs carrying log data (prompt injection via logs).
const DISALLOWED = ['img']

interface MarkdownProps {
  children: string
  onInternalLink?: () => void
}

function buildComponents(onInternalLink?: () => void): Components {
  return {
    pre: ({ children }) => <CodeBlock>{children}</CodeBlock>,
    code: ({ children }) => (
      <code className="rounded-sm bg-raised px-1 py-0.5 font-mono text-[0.88em]">{children}</code>
    ),
    a: ({ href, children }) =>
      href?.startsWith('/') ? (
        <Link
          to={href}
          onClick={onInternalLink}
          className="text-fg underline underline-offset-2 hover:text-muted"
        >
          {children}
        </Link>
      ) : (
        <a
          href={href}
          target="_blank"
          rel="noreferrer"
          className="text-fg underline underline-offset-2 hover:text-muted"
        >
          {children}
        </a>
      ),
    p: ({ children }) => <p className="my-2 leading-relaxed">{children}</p>,
    ul: ({ children }) => <ul className="my-2 list-disc space-y-1 pl-5">{children}</ul>,
    ol: ({ children }) => <ol className="my-2 list-decimal space-y-1 pl-5">{children}</ol>,
    h1: ({ children }) => <h3 className="mt-3 mb-1 font-bold text-fg">{children}</h3>,
    h2: ({ children }) => <h3 className="mt-3 mb-1 font-bold text-fg">{children}</h3>,
    h3: ({ children }) => <h4 className="mt-3 mb-1 font-bold text-fg">{children}</h4>,
  }
}

export function Markdown({ children, onInternalLink }: MarkdownProps) {
  const components = useMemo(() => buildComponents(onInternalLink), [onInternalLink])
  return (
    <div className="text-base break-words text-fg">
      <ReactMarkdown components={components} disallowedElements={DISALLOWED} unwrapDisallowed>
        {children}
      </ReactMarkdown>
    </div>
  )
}
