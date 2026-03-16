export type ExportFormat = 'mp4' | 'mp3' | 'gif'

export type JobStatus =
  | 'queued'
  | 'analyzing'
  | 'downloading'
  | 'processing'
  | 'completed'
  | 'failed'
  | 'canceled'

export type ThemeMode = 'light' | 'dark' | 'system'

export type QualityOption = {
  height?: number
  id: string
  label: string
}

export type VideoAnalysis = {
  availableFormats: ExportFormat[]
  canonicalUrl: string
  durationMs: number
  previewAvailable: boolean
  qualityOptions: QualityOption[]
  sourceId: string
  suggestedFilename: string
  thumbnail: string
  title: string
}

export type PreviewSource = {
  kind: 'remote' | 'local'
  mimeType: string
  url: string
}

export type ClipSelection = {
  endMs: number
  startMs: number
}

export type ClipRequest = {
  filename: string
  format: ExportFormat
  qualityId: string
  selection: ClipSelection
  sourceId: string
  sourceUrl: string
  thumbnail: string
  title: string
}

export type DownloadJob = {
  bytesDownloaded?: number
  bytesTotal?: number
  createdAt: string
  error?: string | null
  etaText?: string | null
  format: ExportFormat
  id: string
  outputFileName?: string | null
  percent: number
  qualityLabel?: string | null
  selection: ClipSelection
  sourceUrl: string
  speedText?: string | null
  status: JobStatus
  targetPath?: string | null
  thumbnail: string
  title: string
  updatedAt: string
}

export type AppSettings = {
  cacheDirectory: string
  defaultFormat: ExportFormat
  defaultQualityId: string
  downloadDirectory: string
  maxConcurrentDownloads: number
  notifyOnComplete: boolean
  openFileAfterDownload: boolean
  rememberClipSelection: boolean
  theme: ThemeMode
}
