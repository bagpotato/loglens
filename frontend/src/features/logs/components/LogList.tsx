import { useEffect, useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useIsDesktop } from '@/shared/hooks/useMediaQuery'
import type { LogEntry } from '@/shared/types/log'
import { LogRow } from './LogRow'

export interface LogListProps {
  entries: LogEntry[]
  selectedId?: string
  focusIndex?: number
  onSelect: (entry: LogEntry) => void
}

const ESTIMATED_ROW_MOBILE_PX = 64
const ESTIMATED_ROW_DESKTOP_PX = 37

/** Virtualized newest-first list; only the rows in view (plus overscan) are mounted. */
export function LogList({ entries, selectedId, focusIndex = -1, onSelect }: LogListProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const isDesktop = useIsDesktop()
  // eslint-disable-next-line react-hooks/incompatible-library -- virtualizer must re-render with fresh measurements
  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => (isDesktop ? ESTIMATED_ROW_DESKTOP_PX : ESTIMATED_ROW_MOBILE_PX),
    getItemKey: (index) => entries[index]?.id ?? index,
    overscan: 12,
    initialRect: { width: 0, height: 800 },
  })

  useEffect(() => {
    if (focusIndex >= 0) virtualizer.scrollToIndex(focusIndex, { align: 'center' })
  }, [focusIndex, virtualizer])

  if (entries.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 items-center justify-center p-8 text-base text-muted">
        No logs match these filters.
      </div>
    )
  }

  return (
    <div
      ref={scrollRef}
      data-testid="log-list"
      className="min-h-0 flex-1 overflow-y-auto overscroll-contain"
    >
      <div role="list" className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((item) => {
          const entry = entries[item.index]
          return (
            <div
              key={item.key}
              role="listitem"
              data-index={item.index}
              ref={virtualizer.measureElement}
              className="absolute top-0 left-0 w-full"
              style={{ transform: `translateY(${item.start}px)` }}
            >
              <LogRow
                entry={entry}
                selected={entry.id === selectedId}
                highlighted={item.index === focusIndex}
                onSelect={onSelect}
              />
            </div>
          )
        })}
      </div>
    </div>
  )
}
