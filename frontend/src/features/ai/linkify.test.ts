import { describe, expect, it } from 'vitest'
import { linkifyTimestamps, logLinkFor } from './linkify'

const HREF =
  '/logs?start=2026-11-21T09%3A58%3A00.000Z&end=2026-11-21T10%3A02%3A00.000Z' +
  '&severity=critical,important,warning,normal&focus=2026-11-21T10%3A00%3A00Z'

describe('logLinkFor', () => {
  it('builds a +-2 minute window focused on the timestamp', () => {
    expect(logLinkFor('2026-11-21T10:00:00Z')).toBe(HREF)
  })

  it('round-trips through URLSearchParams', () => {
    const href = logLinkFor('2026-11-21T12:00:00+02:00') ?? ''
    const params = new URLSearchParams(href.split('?')[1])
    expect(params.get('start')).toBe('2026-11-21T09:58:00.000Z')
    expect(params.get('end')).toBe('2026-11-21T10:02:00.000Z')
    expect(params.get('focus')).toBe('2026-11-21T12:00:00+02:00')
    expect(params.get('severity')).toBe('critical,important,warning,normal')
  })
})

describe('linkifyTimestamps', () => {
  it('turns bracketed RFC3339 timestamps into markdown links', () => {
    expect(linkifyTimestamps('At [2026-11-21T10:00:00Z] sshd failed.')).toBe(
      `At [2026-11-21T10:00:00Z](${HREF}) sshd failed.`,
    )
  })

  it('handles fractional seconds and several citations', () => {
    const out = linkifyTimestamps('[2026-11-21T10:00:00.123Z] and [2026-11-21T11:00:00Z]')
    expect(out.match(/\]\(\/logs\?/g)).toHaveLength(2)
  })

  it('leaves existing links, plain dates and code untouched', () => {
    const text =
      '[2026-11-21T10:00:00Z](/x) on 2026-11-21T10:00:00Z `[2026-11-21T10:00:00Z]`\n' +
      '```\n[2026-11-21T10:00:00Z] log line\n```'
    expect(linkifyTimestamps(text)).toBe(text)
  })

  it('ignores invalid dates', () => {
    expect(linkifyTimestamps('[2026-13-45T99:00:00Z]')).toBe('[2026-13-45T99:00:00Z]')
  })
})
