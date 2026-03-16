import { useEffect, useRef, useState } from 'react'
import { clampNumber, formatTimecode } from '../lib/format'
import type { ClipSelection } from '../types'

type TimelineEditorProps = {
  currentTimeMs: number
  durationMs: number
  onSeek: (nextTimeMs: number) => void
  onSelectionChange: (selection: ClipSelection) => void
  selection: ClipSelection
}

type DragTarget = 'playhead' | 'start' | 'end'

export function clampSelection(
  nextSelection: ClipSelection,
  durationMs: number,
): ClipSelection {
  const safeDuration = Math.max(durationMs, 1)
  const startMs = clampNumber(nextSelection.startMs, 0, safeDuration - 1)
  const endMs = clampNumber(nextSelection.endMs, startMs + 1, safeDuration)
  return { startMs, endMs }
}

function ratio(value: number, durationMs: number) {
  return `${(clampNumber(value, 0, durationMs) / Math.max(durationMs, 1)) * 100}%`
}

export function TimelineEditor({
  currentTimeMs,
  durationMs,
  onSeek,
  onSelectionChange,
  selection,
}: TimelineEditorProps) {
  const [dragTarget, setDragTarget] = useState<DragTarget | null>(null)
  const trackRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    if (!dragTarget) {
      return undefined
    }

    const handlePointerMove = (event: PointerEvent) => {
      const track = trackRef.current
      if (!track) {
        return
      }

      const rect = track.getBoundingClientRect()
      const nextRatio = clampNumber((event.clientX - rect.left) / rect.width, 0, 1)
      const nextTimeMs = Math.round(nextRatio * durationMs)

      if (dragTarget === 'playhead') {
        onSeek(nextTimeMs)
        return
      }

      if (dragTarget === 'start') {
        onSelectionChange(
          clampSelection({ ...selection, startMs: nextTimeMs }, durationMs),
        )
        return
      }

      onSelectionChange(clampSelection({ ...selection, endMs: nextTimeMs }, durationMs))
    }

    const stop = () => setDragTarget(null)

    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', stop)

    return () => {
      window.removeEventListener('pointermove', handlePointerMove)
      window.removeEventListener('pointerup', stop)
    }
  }, [dragTarget, durationMs, onSeek, onSelectionChange, selection])

  const selectionWidth = selection.endMs - selection.startMs

  return (
    <section className="timeline-panel">
      <div className="timeline-metrics">
        <div>
          <p className="eyebrow">Clip Window</p>
          <h3>{formatTimecode(selectionWidth)}</h3>
        </div>
        <div className="timeline-readouts">
          <span>IN {formatTimecode(selection.startMs)}</span>
          <span>NOW {formatTimecode(currentTimeMs)}</span>
          <span>OUT {formatTimecode(selection.endMs)}</span>
        </div>
      </div>

      <div
        className={dragTarget ? 'timeline-track is-dragging' : 'timeline-track'}
        ref={trackRef}
      >
        <button
          aria-label="Seek playhead"
          className={
            dragTarget === 'playhead'
              ? 'timeline-playhead-hitbox active'
              : 'timeline-playhead-hitbox'
          }
          onPointerDown={() => setDragTarget('playhead')}
          style={{ left: ratio(currentTimeMs, durationMs) }}
          type="button"
        />

        <div
          className="timeline-selection"
          style={{
            left: ratio(selection.startMs, durationMs),
            width: ratio(selectionWidth, durationMs),
          }}
        >
          <button
            aria-label="Move clip start"
            className={
              dragTarget === 'start'
                ? 'timeline-handle timeline-handle-start active'
                : 'timeline-handle timeline-handle-start'
            }
            onPointerDown={() => setDragTarget('start')}
            type="button"
          />
          <button
            aria-label="Move clip end"
            className={
              dragTarget === 'end'
                ? 'timeline-handle timeline-handle-end active'
                : 'timeline-handle timeline-handle-end'
            }
            onPointerDown={() => setDragTarget('end')}
            type="button"
          />
        </div>
      </div>
    </section>
  )
}
