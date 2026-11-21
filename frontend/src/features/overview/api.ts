import { useQuery } from '@tanstack/react-query'
import { apiGet } from '@/shared/api/client'
import type { LogEntry, LogsResponse, Stats } from '@/shared/types/log'

export const STATS_RANGE = '24h'
export const LATEST_CRITICAL_LIMIT = 10

export function fetchStats(range: string = STATS_RANGE): Promise<Stats> {
  return apiGet<Stats>('/api/stats', { range })
}

export async function fetchLatestCritical(
  limit: number = LATEST_CRITICAL_LIMIT,
): Promise<LogEntry[]> {
  const response = await apiGet<LogsResponse>('/api/logs', {
    severity: 'critical',
    limit: String(limit),
  })
  return response.entries.slice(0, limit)
}

export function useStats() {
  return useQuery({
    queryKey: ['overview', 'stats', STATS_RANGE],
    queryFn: () => fetchStats(),
    refetchInterval: 60_000,
  })
}

export function useLatestCritical() {
  return useQuery({
    queryKey: ['overview', 'latest-critical', LATEST_CRITICAL_LIMIT],
    queryFn: () => fetchLatestCritical(),
    refetchInterval: 30_000,
  })
}
