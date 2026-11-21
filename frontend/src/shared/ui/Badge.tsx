import type { ReactNode } from 'react'
import { SEVERITY_LABEL, type Severity } from '@/shared/types/severity'

export interface BadgeProps {
  severity?: Severity
  children?: ReactNode
}

export function Badge({ severity, children }: BadgeProps) {
  return (
    <span data-severity={severity} className="ui-badge">
      {children ?? (severity ? SEVERITY_LABEL[severity] : null)}
    </span>
  )
}
