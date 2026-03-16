import { useDeferredValue, useState } from 'react'
import { CloseIcon, FolderIcon, PlayIcon } from '../components/Icons'
import { formatPercent, formatStatusLabel, formatTimecode } from '../lib/format'
import type { DownloadJob } from '../types'

type DownloadsPageProps = {
  busyJobId: string | null
  jobs: DownloadJob[]
  onCancel: (jobId: string) => Promise<void>
  onOpenFolder: (jobId: string) => Promise<void>
  onPlayOutput: (jobId: string) => Promise<void>
  onRemove: (jobId: string) => Promise<void>
}

type FilterKey = 'all' | 'active' | 'completed'

const filters: Array<{ key: FilterKey; label: string }> = [
  { key: 'all', label: 'All Downloads' },
  { key: 'active', label: 'Active' },
  { key: 'completed', label: 'Completed' },
]

const activeStatuses = new Set(['queued', 'analyzing', 'downloading', 'processing'])

export function DownloadsPage({
  busyJobId,
  jobs,
  onCancel,
  onOpenFolder,
  onPlayOutput,
  onRemove,
}: DownloadsPageProps) {
  const [activeFilter, setActiveFilter] = useState<FilterKey>('all')
  const [search, setSearch] = useState('')
  const deferredSearch = useDeferredValue(search)

  const filteredJobs = jobs.filter((job) => {
    const filterMatch =
      activeFilter === 'all'
        ? true
        : activeFilter === 'active'
          ? activeStatuses.has(job.status)
          : job.status === 'completed'

    const needle = deferredSearch.trim().toLowerCase()
    const searchMatch = needle.length === 0 || job.title.toLowerCase().includes(needle)

    return filterMatch && searchMatch
  })

  return (
    <section className="page-section">
      <div className="page-header split">
        <div>
          <h2>Downloads</h2>
        </div>

        <input
          aria-label="Search downloads"
          className="search-input"
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Search clip names..."
          type="search"
          value={search}
        />
      </div>

      <div className="tab-strip">
        {filters.map((filter) => (
          <button
            key={filter.key}
            className={activeFilter === filter.key ? 'tab active' : 'tab'}
            onClick={() => setActiveFilter(filter.key)}
            type="button"
          >
            {filter.label}
          </button>
        ))}
      </div>

      <div className="downloads-list">
        {filteredJobs.length === 0 ? (
          <article className="download-card empty">
            <h3>No matching downloads</h3>
            <p>Clip jobs appear here after you submit them from the workspace.</p>
          </article>
        ) : null}

        {filteredJobs.map((job) => (
          <article key={job.id} className="download-card">
            <img className="download-thumb" src={job.thumbnail} alt={job.title} />
            <div className="download-body">
              <div className="download-header">
                <div>
                  <h3>{job.title}</h3>
                  <p>
                    {job.format.toUpperCase()} • {job.qualityLabel ?? 'Original quality'} •{' '}
                    {formatTimecode(job.selection.endMs - job.selection.startMs)}
                  </p>
                </div>
                <span className={`status-pill status-${job.status}`}>
                  {formatStatusLabel(job.status)}
                </span>
              </div>

              <div className="download-chip-row">
                <span>{job.qualityLabel ?? 'Original quality'}</span>
                <span>{job.format.toUpperCase()}</span>
                <span>{formatTimecode(job.selection.endMs - job.selection.startMs)}</span>
              </div>

              <div className="progress-track">
                <span style={{ width: `${job.percent}%` }} />
              </div>

              <div className="download-footer">
                <p>
                  {formatPercent(job.percent)}
                  {job.speedText ? ` • ${job.speedText}` : ''}
                  {job.etaText ? ` • ETA ${job.etaText}` : ''}
                </p>
                <div className="download-actions">
                  {activeStatuses.has(job.status) ? (
                    <button
                      className="ghost-button danger"
                      disabled={busyJobId === job.id}
                      onClick={() => void onCancel(job.id)}
                      type="button"
                    >
                      Cancel
                    </button>
                  ) : null}
                  {job.status === 'completed' && job.targetPath ? (
                    <button
                      aria-label={`Play ${job.title}`}
                      className="icon-button"
                      disabled={busyJobId === job.id}
                      onClick={() => void onPlayOutput(job.id)}
                      title="Play clip"
                      type="button"
                    >
                      <PlayIcon title="Play clip" />
                    </button>
                  ) : null}
                  {job.status === 'completed' ? (
                    <button
                      aria-label={`Open folder for ${job.title}`}
                      className="icon-button"
                      disabled={busyJobId === job.id}
                      onClick={() => void onOpenFolder(job.id)}
                      title="Open folder"
                      type="button"
                    >
                      <FolderIcon title="Open folder" />
                    </button>
                  ) : null}
                  {['completed', 'failed', 'canceled'].includes(job.status) ? (
                    <button
                      aria-label={`Remove ${job.title}`}
                      className="icon-button danger"
                      disabled={busyJobId === job.id}
                      onClick={() => void onRemove(job.id)}
                      title="Remove from list"
                      type="button"
                    >
                      <CloseIcon title="Remove from list" />
                    </button>
                  ) : null}
                </div>
              </div>

              {job.error ? <p className="inline-message error">{job.error}</p> : null}
            </div>
          </article>
        ))}
      </div>
    </section>
  )
}
