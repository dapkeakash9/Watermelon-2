import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { DownloadJob } from '../types'
import { DownloadsPage } from './DownloadsPage'

const jobs: DownloadJob[] = [
  {
    createdAt: '2026-03-12T10:00:00Z',
    format: 'mp4',
    id: '1',
    percent: 42,
    selection: { endMs: 28_000, startMs: 4_000 },
    sourceUrl: 'https://youtube.com/watch?v=1',
    status: 'downloading',
    thumbnail: 'https://example.com/thumb-1.jpg',
    title: 'Rickroll Clip',
    updatedAt: '2026-03-12T10:01:00Z',
  },
  {
    createdAt: '2026-03-12T11:00:00Z',
    format: 'gif',
    id: '2',
    percent: 100,
    selection: { endMs: 13_000, startMs: 2_000 },
    sourceUrl: 'https://youtube.com/watch?v=2',
    status: 'completed',
    targetPath: 'C:/Downloads/clip.gif',
    thumbnail: 'https://example.com/thumb-2.jpg',
    title: 'Reaction Loop',
    updatedAt: '2026-03-12T11:02:00Z',
  },
]

describe('DownloadsPage', () => {
  it('filters by completion state and search text', async () => {
    const user = userEvent.setup()

    render(
      <DownloadsPage
        busyJobId={null}
        jobs={jobs}
        onCancel={vi.fn(async () => undefined)}
        onOpenFolder={vi.fn(async () => undefined)}
        onPlayOutput={vi.fn(async () => undefined)}
        onRemove={vi.fn(async () => undefined)}
      />,
    )

    await user.click(screen.getByRole('button', { name: 'Completed' }))
    await user.type(screen.getByRole('searchbox', { name: 'Search downloads' }), 'reaction')

    expect(screen.getByText('Reaction Loop')).toBeInTheDocument()
    expect(screen.queryByText('Rickroll Clip')).not.toBeInTheDocument()
  })
})
