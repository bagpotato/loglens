import type { Severity } from './severity'

export interface LogFilters {
  host?: string
  source?: string
  severity: Severity[]
  q?: string
  start?: string
  end?: string
  limit?: number
}
