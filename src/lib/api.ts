import type {
  AppSettings,
  ClipRequest,
  DownloadJob,
  PreviewSource,
  VideoAnalysis,
} from '../types'

type ApiErrorPayload = {
  message?: string
}

async function apiRequest<T>(input: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`/api${input}`, {
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
    ...init,
  })

  if (!response.ok) {
    let payload: ApiErrorPayload | null = null
    try {
      payload = (await response.json()) as ApiErrorPayload
    } catch {
      payload = null
    }

    throw new Error(payload?.message || `Request failed with status ${response.status}.`)
  }

  if (response.status === 204) {
    return undefined as T
  }

  return (await response.json()) as T
}

export function analyzeYoutubeUrl(url: string) {
  return apiRequest<VideoAnalysis>('/analyze', {
    method: 'POST',
    body: JSON.stringify({ url }),
  })
}

export function getPreviewSource(source: string) {
  return apiRequest<PreviewSource>('/preview', {
    method: 'POST',
    body: JSON.stringify({ source }),
  })
}

export function createClipJob(request: ClipRequest) {
  return apiRequest<DownloadJob>('/jobs', {
    method: 'POST',
    body: JSON.stringify(request),
  })
}

export function listJobs() {
  return apiRequest<DownloadJob[]>('/jobs')
}

export function getJob(jobId: string) {
  return apiRequest<DownloadJob>(`/jobs/${jobId}`)
}

export function cancelJob(jobId: string) {
  return apiRequest<void>(`/jobs/${jobId}/cancel`, {
    method: 'POST',
  })
}

export function removeJob(jobId: string) {
  return apiRequest<void>(`/jobs/${jobId}`, {
    method: 'DELETE',
  })
}

export function openOutput(jobId: string) {
  return apiRequest<void>(`/jobs/${jobId}/open`, {
    method: 'POST',
  })
}

export function playOutput(jobId: string) {
  return apiRequest<void>(`/jobs/${jobId}/play`, {
    method: 'POST',
  })
}

export function openOutputFolder(jobId: string) {
  return apiRequest<void>(`/jobs/${jobId}/folder`, {
    method: 'POST',
  })
}

export function getSettings() {
  return apiRequest<AppSettings>('/settings')
}

export function saveSettings(settings: AppSettings) {
  return apiRequest<AppSettings>('/settings', {
    method: 'PUT',
    body: JSON.stringify(settings),
  })
}

export function pickDownloadDirectory() {
  return apiRequest<string | null>('/settings/pick-download-directory', {
    method: 'POST',
  })
}

export function clearCache() {
  return apiRequest<number>('/cache/clear', {
    method: 'POST',
  })
}
