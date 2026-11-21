import type { Severity, SeveritySource } from './severity'

export interface LogEntry {
  id: string
  timestamp: string
  host: string
  source: string
  container?: string
  unit?: string
  message: string
  severity: Severity
  severity_source: SeveritySource
  reason?: string
}

export interface LogsResponse {
  entries: LogEntry[]
}

export interface Meta {
  hosts: string[]
  sources: string[]
}

export interface Stats {
  by_severity: Record<string, number>
  by_source: Record<string, number>
}
