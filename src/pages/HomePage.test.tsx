import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { UseAppModel } from '../hooks/useAppModel'
import { HomePage } from './HomePage'

function buildModel(): UseAppModel {
  return {
    analysis: {
      availableFormats: ['mp4', 'mp3', 'gif'],
      canonicalUrl: 'https://youtube.com/watch?v=dQw4w9WgXcQ',
      durationMs: 285_000,
      previewAvailable: true,
      qualityOptions: [
        { id: '1080p', label: '1080p Full HD' },
        { id: '720p', label: '720p HD' },
      ],
      sourceId: 'dQw4w9WgXcQ',
      suggestedFilename: 'Rick_Astley_clip',
      thumbnail: 'https://example.com/thumb.jpg',
      title: 'Rick Astley',
    },
    analyzeBusy: false,
    analyzeUrl: vi.fn(async () => undefined),
    busyJobId: null,
    cacheClearing: false,
    cancelJob: vi.fn(async () => undefined),
    clearCache: vi.fn(async () => undefined),
    defaultFormat: 'mp4',
    defaultQualityId: '1080p',
    dependencyMessage: null,
    enqueueClipJob: vi.fn(async () => undefined),
    jobs: [],
    openOutput: vi.fn(async () => undefined),
    openOutputFolder: vi.fn(async () => undefined),
    pasteFromClipboard: vi.fn(async () => undefined),
    pickDownloadDirectory: vi.fn(async () => null),
    playOutput: vi.fn(async () => undefined),
    previewSource: {
      kind: 'remote',
      mimeType: 'video/mp4',
      url: 'https://example.com/video.mp4',
    },
    removeJob: vi.fn(async () => undefined),
    saveBusy: false,
    saveSettings: vi.fn(async () => undefined),
    selection: { startMs: 5_000, endMs: 15_000 },
    setSelection: vi.fn(),
    settings: {
      cacheDirectory: '',
      defaultFormat: 'mp4',
      defaultQualityId: '1080p',
      downloadDirectory: 'C:/Downloads',
      maxConcurrentDownloads: 2,
      notifyOnComplete: true,
      openFileAfterDownload: false,
      rememberClipSelection: true,
      theme: 'dark',
    },
    setSettings: vi.fn(),
    setUrl: vi.fn(),
    setVideoError: vi.fn(),
    url: 'https://youtube.com/watch?v=dQw4w9WgXcQ',
    urlError: null,
    videoError: null,
  }
}

describe('HomePage', () => {
  it('keeps export format selection exclusive', async () => {
    const user = userEvent.setup()
    render(<HomePage model={buildModel()} />)

    const mp4Button = screen.getByRole('button', { name: 'MP4' })
    const gifButton = screen.getByRole('button', { name: 'GIF' })

    expect(mp4Button.className).toContain('active')
    await user.click(gifButton)

    expect(gifButton.className).toContain('active')
    expect(mp4Button.className).not.toContain('active')
  })
})
