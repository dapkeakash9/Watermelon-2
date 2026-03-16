import { startTransition, useEffect, useState } from 'react'
import {
  analyzeYoutubeUrl,
  cancelJob as cancelJobRequest,
  clearCache as clearCacheRequest,
  createClipJob,
  getPreviewSource,
  getSettings,
  listJobs,
  openOutputFolder as openOutputFolderRequest,
  openOutput as openOutputRequest,
  playOutput as playOutputRequest,
  pickDownloadDirectory as pickDownloadDirectoryRequest,
  removeJob as removeJobRequest,
  saveSettings as saveSettingsRequest,
} from '../lib/api'
import { clampNumber, sanitizeFilenameStem } from '../lib/format'
import type {
  AppSettings,
  ClipRequest,
  ClipSelection,
  DownloadJob,
  PreviewSource,
  VideoAnalysis,
} from '../types'

const DEFAULT_SELECTION_MS = 30_000

function defaultSettings(): AppSettings {
  return {
    cacheDirectory: '',
    defaultFormat: 'mp4',
    defaultQualityId: '1080p',
    downloadDirectory: '',
    maxConcurrentDownloads: 2,
    notifyOnComplete: true,
    openFileAfterDownload: false,
    rememberClipSelection: true,
    theme: 'dark',
  }
}

function defaultSelection(durationMs: number): ClipSelection {
  return {
    startMs: 0,
    endMs: clampNumber(durationMs, 1, DEFAULT_SELECTION_MS),
  }
}

export type UseAppModel = ReturnType<typeof useAppModel>

export function useAppModel() {
  const [analysis, setAnalysis] = useState<VideoAnalysis | null>(null)
  const [analyzeBusy, setAnalyzeBusy] = useState(false)
  const [busyJobId, setBusyJobId] = useState<string | null>(null)
  const [cacheClearing, setCacheClearing] = useState(false)
  const [dependencyMessage, setDependencyMessage] = useState<string | null>(null)
  const [jobs, setJobs] = useState<DownloadJob[]>([])
  const [previewSource, setPreviewSource] = useState<PreviewSource | null>(null)
  const [saveBusy, setSaveBusy] = useState(false)
  const [selection, setSelection] = useState<ClipSelection>({ startMs: 0, endMs: 1 })
  const [settings, setSettings] = useState<AppSettings>(defaultSettings())
  const [url, setUrl] = useState('')
  const [urlError, setUrlError] = useState<string | null>(null)
  const [videoError, setVideoError] = useState<string | null>(null)

  useEffect(() => {
    let disposed = false

    const loadInitial = async () => {
      try {
        const [storedSettings, storedJobs] = await Promise.all([getSettings(), listJobs()])
        if (disposed) {
          return
        }
        setSettings(storedSettings)
        startTransition(() => setJobs(storedJobs))
      } catch (error) {
        if (!disposed) {
          setDependencyMessage(
            error instanceof Error ? error.message : 'Unable to initialize Watermelon 2.',
          )
        }
      }
    }

    void loadInitial()

    const interval = window.setInterval(() => {
      void listJobs()
        .then((storedJobs) => {
          if (!disposed) {
            startTransition(() => setJobs(storedJobs))
          }
        })
        .catch(() => {
          if (!disposed) {
            setDependencyMessage('Unable to refresh download queue state.')
          }
        })
    }, 1500)

    return () => {
      disposed = true
      window.clearInterval(interval)
    }
  }, [])

  useEffect(() => {
    document.documentElement.dataset.theme = settings.theme
  }, [settings.theme])

  const analyzeUrl = async () => {
    const trimmedUrl = url.trim()
    if (!trimmedUrl) {
      setUrlError('Enter a YouTube video URL.')
      return
    }

    setAnalyzeBusy(true)
    setUrlError(null)
    setVideoError(null)
    setDependencyMessage(null)

    try {
      const nextAnalysis = await analyzeYoutubeUrl(trimmedUrl)
      const nextPreview = await getPreviewSource(nextAnalysis.canonicalUrl)
      setAnalysis(nextAnalysis)
      setPreviewSource(nextPreview)
      setSelection(defaultSelection(nextAnalysis.durationMs))
      setUrl(nextAnalysis.canonicalUrl)
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to analyze the video URL.'
      setUrlError(message)
      if (/yt-dlp|ffmpeg/i.test(message)) {
        setDependencyMessage(message)
      }
    } finally {
      setAnalyzeBusy(false)
    }
  }

  const pasteFromClipboard = async () => {
    try {
      const text = await navigator.clipboard.readText()
      setUrl(text)
      setUrlError(null)
    } catch {
      setUrlError('Clipboard read failed. Paste the URL manually.')
    }
  }

  const refreshJobs = async () => {
    setJobs(await listJobs())
  }

  const enqueueClipJob = async (
    request: Omit<ClipRequest, 'filename'> & { filename?: string },
  ) => {
    const filename = sanitizeFilenameStem(
      request.filename?.trim() || analysis?.suggestedFilename || request.title,
    )
    await createClipJob({ ...request, filename })
    await refreshJobs()
  }

  const cancelJob = async (jobId: string) => {
    setBusyJobId(jobId)
    try {
      await cancelJobRequest(jobId)
      await refreshJobs()
    } finally {
      setBusyJobId(null)
    }
  }

  const removeJob = async (jobId: string) => {
    setBusyJobId(jobId)
    try {
      await removeJobRequest(jobId)
      await refreshJobs()
    } finally {
      setBusyJobId(null)
    }
  }

  const openOutput = async (jobId: string) => {
    setBusyJobId(jobId)
    try {
      await openOutputRequest(jobId)
    } finally {
      setBusyJobId(null)
    }
  }

  const playOutput = async (jobId: string) => {
    setBusyJobId(jobId)
    try {
      await playOutputRequest(jobId)
    } finally {
      setBusyJobId(null)
    }
  }

  const openOutputFolder = async (jobId: string) => {
    setBusyJobId(jobId)
    try {
      await openOutputFolderRequest(jobId)
    } finally {
      setBusyJobId(null)
    }
  }

  const saveSettings = async (nextSettings: AppSettings) => {
    setSaveBusy(true)
    try {
      const stored = await saveSettingsRequest(nextSettings)
      setSettings(stored)
    } finally {
      setSaveBusy(false)
    }
  }

  const pickDownloadDirectory = async () => {
    const nextPath = await pickDownloadDirectoryRequest()
    if (nextPath) {
      setSettings((current) => ({ ...current, downloadDirectory: nextPath }))
    }
    return nextPath
  }

  const clearCache = async () => {
    setCacheClearing(true)
    try {
      await clearCacheRequest()
      setSettings(await getSettings())
    } finally {
      setCacheClearing(false)
    }
  }

  const defaultFormat =
    analysis?.availableFormats.includes(settings.defaultFormat) === true
      ? settings.defaultFormat
      : analysis?.availableFormats[0] ?? 'mp4'

  const defaultQualityId =
    analysis?.qualityOptions.some((option) => option.id === settings.defaultQualityId)
      ? settings.defaultQualityId
      : analysis?.qualityOptions[0]?.id ?? '1080p'

  return {
    analysis,
    analyzeBusy,
    analyzeUrl,
    busyJobId,
    cacheClearing,
    cancelJob,
    clearCache,
    defaultFormat,
    defaultQualityId,
    dependencyMessage,
    enqueueClipJob,
    jobs,
    openOutput,
    openOutputFolder,
    pasteFromClipboard,
    pickDownloadDirectory,
    playOutput,
    previewSource,
    removeJob,
    saveBusy,
    saveSettings,
    selection,
    setSelection,
    settings,
    setSettings,
    setUrl,
    setVideoError,
    url,
    urlError,
    videoError,
  }
}
