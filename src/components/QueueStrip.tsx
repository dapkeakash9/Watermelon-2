import { FolderIcon, PlayIcon } from './Icons'
import { formatPercent, formatStatusLabel, formatTimecode } from '../lib/format'
import type { DownloadJob } from '../types'

type QueueStripProps = {
  busyJobId: string | null
  jobs: DownloadJob[]
  onCancel: (jobId: string) => Promise<void>
  onOpenFolder: (jobId: string) => Promise<void>
  onPlayOutput: (jobId: string) => Promise<void>
}

const activeStatuses = new Set(['queued', 'analyzing', 'downloading', 'processing'])

export function QueueStrip({
  busyJobId,
  jobs,
  onCancel,
  onOpenFolder,
  onPlayOutput,
}: QueueStripProps) {
  const activeJobs = jobs.filter((job) => activeStatuses.has(job.status))
  const completedJobs = jobs.filter((job) => job.status === 'completed')
  const visibleJobs = [...activeJobs, ...completedJobs].slice(0, 3)

  return (
    <section className="queue-strip">
      <div className="queue-strip-header">
        <h2>{activeJobs.length > 0 ? `${activeJobs.length} active` : 'Idle'}</h2>
      </div>

      <div className="queue-strip-grid">
        {visibleJobs.length === 0 ? (
          <article className="queue-card empty">
            <p>No clip jobs yet.</p>
            <span>Analyze a video, mark your range, and download the clip.</span>
          </article>
        ) : null}

        {visibleJobs.map((job) => (
          <article key={job.id} className="queue-card">
            <img className="queue-thumb" src={job.thumbnail} alt={job.title} />
            <div className="queue-body">
              <div className="queue-row">
                <h3>{job.title}</h3>
                <span className={`status-pill status-${job.status}`}>
                  {formatStatusLabel(job.status)}
                </span>
              </div>
              <p className="queue-subtitle">
                {job.format.toUpperCase()} clip {formatTimecode(job.selection.endMs - job.selection.startMs)}
              </p>
              <div className="progress-track compact">
                <span style={{ width: `${job.percent}%` }} />
              </div>
              <div className="queue-row">
                <p>{formatPercent(job.percent)}</p>
                <div className="queue-actions">
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
                </div>
              </div>
            </div>
          </article>
        ))}
      </div>
    </section>
  )
}
