import { useEffect, useRef, useState } from 'react'
import { ThemedSelect } from '../components/ThemedSelect'
import { TimelineEditor, clampSelection } from '../components/TimelineEditor'
import type { UseAppModel } from '../hooks/useAppModel'
import { formatTimecode } from '../lib/format'
import type { ExportFormat } from '../types'

type HomePageProps = {
  model: UseAppModel
}

const formatLabels: Record<ExportFormat, string> = {
  gif: 'GIF',
  mp3: 'MP3',
  mp4: 'MP4',
}

export function HomePage({ model }: HomePageProps) {
  const videoRef = useRef<HTMLVideoElement | null>(null)
  const [currentTimeMs, setCurrentTimeMs] = useState(0)
  const [exportFormat, setExportFormat] = useState<ExportFormat>(model.defaultFormat)
  const [qualityId, setQualityId] = useState(model.defaultQualityId)
  const [filename, setFilename] = useState('')
  const clipDurationMs = model.selection.endMs - model.selection.startMs
  const selectedQualityLabel =
    model.analysis?.qualityOptions.find((option) => option.id === qualityId)?.label ??
    'No quality selected'
  const qualityOptions =
    model.analysis?.qualityOptions.map((option) => ({
      label: option.label,
      value: option.id,
    })) ?? []

  useEffect(() => {
    setExportFormat(model.defaultFormat)
  }, [model.defaultFormat])

  useEffect(() => {
    setQualityId(model.defaultQualityId)
  }, [model.defaultQualityId])

  useEffect(() => {
    if (model.analysis) {
      setFilename(model.analysis.suggestedFilename)
      setCurrentTimeMs(0)
    }
  }, [model.analysis])

  const togglePlayback = async () => {
    const video = videoRef.current
    if (!video) {
      return
    }
    if (video.paused) {
      await video.play()
      return
    }
    video.pause()
  }

  const handleSeek = (nextTimeMs: number) => {
    setCurrentTimeMs(nextTimeMs)
    const video = videoRef.current
    if (video) {
      video.currentTime = nextTimeMs / 1000
    }
  }

  const applyCurrentTimeTo = (target: 'start' | 'end') => {
    if (!model.analysis) {
      return
    }
    const nextSelection = clampSelection(
      {
        ...model.selection,
        [target === 'start' ? 'startMs' : 'endMs']: currentTimeMs,
      },
      model.analysis.durationMs,
    )
    model.setSelection(nextSelection)
  }

  const submitClip = async () => {
    if (!model.analysis) {
      return
    }
    await model.enqueueClipJob({
      filename,
      format: exportFormat,
      qualityId,
      selection: model.selection,
      sourceId: model.analysis.sourceId,
      sourceUrl: model.analysis.canonicalUrl,
      thumbnail: model.analysis.thumbnail,
      title: model.analysis.title,
    })
  }

  const canDownload =
    Boolean(model.analysis) &&
    model.selection.endMs > model.selection.startMs &&
    filename.trim().length > 0

  return (
    <section className="workspace-grid">
      <div className="workspace-main">
        <article className="video-card">
          <div className="studio-header">
            <div>
              <h3>{model.analysis?.title ?? 'Ready for a source video'}</h3>
              <p className="page-copy compact-copy">
                {model.analysis
                  ? 'Trim the exact range and export it directly.'
                  : 'Analyze a YouTube link to load a playable preview.'}
              </p>
            </div>
            <div className="studio-chips">
              <span>{selectedQualityLabel}</span>
              <span>{formatLabels[exportFormat]}</span>
              <span>{formatTimecode(clipDurationMs)}</span>
            </div>
          </div>

          {model.previewSource ? (
            <video
              className="video-player"
              controls={false}
              onError={() =>
                model.setVideoError(
                  'Preview playback failed. Re-analyze the video or try another source.',
                )
              }
              onLoadedMetadata={(event) => {
                const durationMs = Math.round(event.currentTarget.duration * 1000)
                model.setSelection(clampSelection(model.selection, durationMs))
              }}
              onTimeUpdate={(event) => {
                setCurrentTimeMs(Math.round(event.currentTarget.currentTime * 1000))
              }}
              poster={model.analysis?.thumbnail}
              ref={videoRef}
              src={model.previewSource.url}
            />
          ) : (
            <div className="video-placeholder">
              <div>
                <h3>Video preview</h3>
                <p>Analyze a YouTube link to load a playable preview and timeline.</p>
              </div>
            </div>
          )}

          <div className="timeline-toolbar">
            <div className="video-controls">
              <button className="secondary-button" onClick={() => void togglePlayback()} type="button">
                Play / Pause
              </button>
              <button className="ghost-button" onClick={() => applyCurrentTimeTo('start')} type="button">
                Set IN
              </button>
              <button className="ghost-button" onClick={() => applyCurrentTimeTo('end')} type="button">
                Set OUT
              </button>
            </div>

            <div className="video-meta-inline">
              <span>Now {formatTimecode(currentTimeMs)}</span>
              <span>Total {formatTimecode(model.analysis?.durationMs ?? 0)}</span>
            </div>
          </div>

          {model.analysis ? (
            <TimelineEditor
              currentTimeMs={currentTimeMs}
              durationMs={model.analysis.durationMs}
              onSeek={handleSeek}
              onSelectionChange={model.setSelection}
              selection={model.selection}
            />
          ) : null}
        </article>

        {model.videoError ? <p className="inline-message error">{model.videoError}</p> : null}
      </div>

      <aside className="workspace-panel">
        <div className="panel-header">
          <p className="eyebrow">Export</p>
          <h3>{model.analysis?.title ?? 'No video loaded'}</h3>
          <p className="page-copy">
            {model.analysis
              ? `Total duration ${formatTimecode(model.analysis.durationMs)}`
              : 'Select a clip after loading a video.'}
          </p>
        </div>

        <div className="selection-summary premium">
          <div>
            <span>IN</span>
            <strong>{formatTimecode(model.selection.startMs)}</strong>
          </div>
          <div>
            <span>OUT</span>
            <strong>{formatTimecode(model.selection.endMs)}</strong>
          </div>
          <div>
            <span>Clip</span>
            <strong>{formatTimecode(clipDurationMs)}</strong>
          </div>
        </div>

        <div className="form-section premium-block">
          <label className="field-label">Format</label>
          <div className="choice-grid">
            {model.analysis?.availableFormats.map((format) => (
              <button
                key={format}
                className={exportFormat === format ? 'choice-card active' : 'choice-card'}
                onClick={() => setExportFormat(format)}
                type="button"
              >
                {formatLabels[format]}
              </button>
            ))}
          </div>
        </div>

        <div className="form-section premium-block">
          <label className="field-label" htmlFor="quality">
            Quality
          </label>
          <ThemedSelect
            id="quality"
            onChange={setQualityId}
            options={qualityOptions}
            value={qualityId}
          />
        </div>

        <div className="form-section premium-block">
          <label className="field-label" htmlFor="filename">
            Filename
          </label>
          <input
            id="filename"
            onChange={(event) => setFilename(event.target.value)}
            type="text"
            value={filename}
          />
        </div>

        <button
          className="primary-button"
          disabled={!canDownload}
          onClick={() => void submitClip()}
          type="button"
        >
          Download Clip
        </button>
      </aside>
    </section>
  )
}
