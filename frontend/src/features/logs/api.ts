import { apiGet, type QueryParams } from '@/shared/api/client'
import type { LogFilters } from '@/shared/types/filters'
import type { LogsResponse, Meta } from '@/shared/types/log'

export const DEFAULT_LOG_LIMIT = 2000

/** Query params shared by `GET /api/logs` and the live tail (which ignores the time range). */
export function tailParams(filters: LogFilters): QueryParams {
  return {
    host: filters.host,
    source: filters.source,
    severity: filters.severity.length > 0 ? filters.severity.join(',') : undefined,
    q: filters.q,
  }
}

export function logsParams(filters: LogFilters): QueryParams {
  return {
    ...tailParams(filters),
    start: filters.start,
    end: filters.end,
    limit: String(filters.limit ?? DEFAULT_LOG_LIMIT),
  }
}

export function fetchLogs(filters: LogFilters): Promise<LogsResponse> {
  return apiGet<LogsResponse>('/api/logs', logsParams(filters))
}

export function fetchMeta(): Promise<Meta> {
  return apiGet<Meta>('/api/meta')
}
