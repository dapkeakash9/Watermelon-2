import { useEffect, useState } from 'react'
import { ThemedSelect } from '../components/ThemedSelect'
import type { AppSettings, ExportFormat, ThemeMode } from '../types'

type SettingsPageProps = {
  cacheClearing: boolean
  onBrowseDirectory: () => Promise<string | null>
  onClearCache: () => Promise<void>
  onSave: (settings: AppSettings) => Promise<void>
  saveBusy: boolean
  settings: AppSettings
}

const themes: ThemeMode[] = ['light', 'dark', 'system']
const formats: ExportFormat[] = ['mp4', 'mp3', 'gif']
const qualityPresets = ['2160p', '1440p', '1080p', '720p', '480p', '360p', 'best']

export function SettingsPage({
  cacheClearing,
  onBrowseDirectory,
  onClearCache,
  onSave,
  saveBusy,
  settings,
}: SettingsPageProps) {
  const [draft, setDraft] = useState(settings)
  const formatOptions = formats.map((format) => ({
    label: format.toUpperCase(),
    value: format,
  }))
  const qualityOptions = qualityPresets.map((quality) => ({
    label: quality === 'best' ? 'Best available' : quality,
    value: quality,
  }))
  const behaviorOptions = [
    {
      key: 'notifyOnComplete' as const,
      title: 'Show completion notifications',
      description: 'Display a Windows notification when a clip export finishes.',
    },
    {
      key: 'openFileAfterDownload' as const,
      title: 'Open file after download',
      description: 'Launch the finished clip in your default media app automatically.',
    },
    {
      key: 'rememberClipSelection' as const,
      title: 'Remember clip selection',
      description: 'Keep the last IN and OUT range when you re-open the same source.',
    },
  ]

  useEffect(() => {
    setDraft(settings)
  }, [settings])

  const isDirty = JSON.stringify(draft) !== JSON.stringify(settings)

  return (
    <section className="page-section">
      <div className="page-header">
        <h2>Settings</h2>
      </div>

      <div className="settings-layout">
        <article className="settings-card">
          <h3>Appearance</h3>
          <div className="choice-grid triple">
            {themes.map((theme) => (
              <button
                key={theme}
                className={draft.theme === theme ? 'choice-card active' : 'choice-card'}
                onClick={() => setDraft((current) => ({ ...current, theme }))}
                type="button"
              >
                {theme}
              </button>
            ))}
          </div>
        </article>

        <article className="settings-card">
          <h3>Download location</h3>
          <div className="inline-field">
            <input readOnly type="text" value={draft.downloadDirectory} />
            <button
              className="ghost-button"
              onClick={() =>
                void onBrowseDirectory().then((nextPath) => {
                  if (nextPath) {
                    setDraft((current) => ({ ...current, downloadDirectory: nextPath }))
                  }
                })
              }
              type="button"
            >
              Browse
            </button>
          </div>
        </article>

        <article className="settings-card">
          <h3>Default export</h3>
          <div className="form-grid">
            <label>
              <span>Format</span>
              <ThemedSelect
                onChange={(value) =>
                  setDraft((current) => ({ ...current, defaultFormat: value as ExportFormat }))
                }
                options={formatOptions}
                value={draft.defaultFormat}
              />
            </label>

            <label>
              <span>Quality</span>
              <ThemedSelect
                onChange={(value) =>
                  setDraft((current) => ({ ...current, defaultQualityId: value }))
                }
                options={qualityOptions}
                value={draft.defaultQualityId}
              />
            </label>
          </div>
        </article>

        <article className="settings-card">
          <h3>Download behavior</h3>
          <div className="behavior-list">
            {behaviorOptions.map((option) => (
              <label key={option.key} className="behavior-row">
                <div className="behavior-copy">
                  <span>{option.title}</span>
                  <small>{option.description}</small>
                </div>
                <input
                  checked={draft[option.key]}
                  onChange={(event) =>
                    setDraft((current) => ({
                      ...current,
                      [option.key]: event.target.checked,
                    }))
                  }
                  type="checkbox"
                />
              </label>
            ))}
          </div>
        </article>

        <article className="settings-card">
          <h3>Performance</h3>
          <label className="slider-field">
            <span>Max concurrent downloads: {draft.maxConcurrentDownloads}</span>
            <input
              max={5}
              min={1}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  maxConcurrentDownloads: Number(event.target.value),
                }))
              }
              type="range"
              value={draft.maxConcurrentDownloads}
            />
          </label>
        </article>

        <article className="settings-card danger-zone">
          <h3>Cache</h3>
          <p>{draft.cacheDirectory || 'Cache directory will be created on first use.'}</p>
          <button
            className="ghost-button danger"
            disabled={cacheClearing}
            onClick={() => void onClearCache()}
            type="button"
          >
            {cacheClearing ? 'Clearing...' : 'Clear cache'}
          </button>
        </article>
      </div>

      <div className="page-actions">
        <button
          className="ghost-button"
          disabled={!isDirty}
          onClick={() => setDraft(settings)}
          type="button"
        >
          Discard Changes
        </button>
        <button
          className="primary-button"
          disabled={!isDirty || saveBusy}
          onClick={() => void onSave(draft)}
          type="button"
        >
          {saveBusy ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </section>
  )
}
