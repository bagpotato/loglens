import { keepPreviousData, useQuery } from '@tanstack/react-query'
import type { LogFilters } from '@/shared/types/filters'
import { fetchLogs, fetchMeta } from './api'

export function useLogs(filters: LogFilters) {
  return useQuery({
    queryKey: ['logs', filters],
    queryFn: () => fetchLogs(filters),
    placeholderData: keepPreviousData,
  })
}

export function useMeta() {
  return useQuery({ queryKey: ['meta'], queryFn: fetchMeta, staleTime: 5 * 60_000 })
}
