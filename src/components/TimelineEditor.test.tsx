import { describe, expect, it } from 'vitest'
import { clampSelection } from './TimelineEditor'

describe('clampSelection', () => {
  it('keeps start before end', () => {
    expect(clampSelection({ startMs: 9_000, endMs: 4_000 }, 10_000)).toEqual({
      startMs: 9_000,
      endMs: 9_001,
    })
  })

  it('keeps values inside duration', () => {
    expect(clampSelection({ startMs: -100, endMs: 15_000 }, 10_000)).toEqual({
      startMs: 0,
      endMs: 10_000,
    })
  })
})
