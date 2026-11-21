import { describe, expect, it } from 'vitest'
import type { LogEntry } from '@/shared/types/log'
import { closestIndex, mergeEntries, prependEntries } from './entries'

function entry(id: string, timestamp = '2026-11-21T10:00:00Z'): LogEntry {
  return {
    id,
    timestamp,
    host: 'web1',
    source: 'ssh',
    message: id,
    severity: 'normal',
    severity_source: 'default',
  }
}

describe('entries helpers', () => {
  it('prepends newest first and caps the buffer', () => {
    const next = prependEntries([entry('a')], [entry('b'), entry('c')], 2)
    expect(next.map((e) => e.id)).toEqual(['c', 'b'])
  })

  it('merges live and queried entries without duplicates', () => {
    const merged = mergeEntries([entry('x'), entry('a')], [entry('a'), entry('b')])
    expect(merged.map((e) => e.id)).toEqual(['x', 'a', 'b'])
  })

  it('finds the entry closest to a timestamp', () => {
    const list = [
      entry('3', '2026-11-21T10:02:00Z'),
      entry('2', '2026-11-21T10:01:00Z'),
      entry('1', '2026-11-21T10:00:00Z'),
    ]
    expect(closestIndex(list, '2026-11-21T10:01:10Z')).toBe(1)
    expect(closestIndex(list, 'nope')).toBe(-1)
  })
})
