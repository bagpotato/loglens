export type Severity = 'critical' | 'important' | 'warning' | 'normal'

export type SeveritySource = 'rule' | 'ai' | 'default'

export const SEVERITIES: readonly Severity[] = ['critical', 'important', 'warning', 'normal']

export const SEVERITY_LABEL: Record<Severity, string> = {
  critical: 'Critical',
  important: 'Important',
  warning: 'Warning',
  normal: 'Normal',
}

export function isSeverity(value: string): value is Severity {
  return (SEVERITIES as readonly string[]).includes(value)
}
