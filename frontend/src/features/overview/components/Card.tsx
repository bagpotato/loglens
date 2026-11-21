import type { ReactNode } from 'react'

export interface CardProps {
  title: string
  action?: ReactNode
  children?: ReactNode
  className?: string
}

export function Card({ title, action, children, className = '' }: CardProps) {
  return (
    <section aria-label={title} className={`ui-panel ${className}`}>
      <header className="ui-panel-head">
        <h2 className="text-base font-bold tracking-tight text-fg">{title}</h2>
        {action}
      </header>
      <div className="ui-panel-body">{children}</div>
    </section>
  )
}

export function CardMessage({ children }: { children: ReactNode }) {
  return <p className="py-6 text-center text-base text-subtle">{children}</p>
}
