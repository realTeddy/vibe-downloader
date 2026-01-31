import type {
  DownloadRecord,
  DownloadStats,
  FileTypeConfig,
  Settings,
  AddDownloadRequest,
  AddDownloadResponse,
} from '../types'

const BASE_URL = '/api'

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`, {
    headers: {
      'Content-Type': 'application/json',
    },
    ...options,
  })
  
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }))
    throw new Error(error.error || `HTTP ${response.status}`)
  }
  
  // Handle empty responses (204 No Content)
  if (response.status === 204) {
    return undefined as T
  }
  
  return response.json()
}

export const api = {
  // Downloads
  getDownloads: () => fetchJson<DownloadRecord[]>('/downloads'),
  
  addDownload: (data: AddDownloadRequest) =>
    fetchJson<AddDownloadResponse>('/downloads', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  
  removeDownload: (id: string) =>
    fetchJson<void>(`/downloads/${id}`, { method: 'DELETE' }),
  
  cancelDownload: (id: string) =>
    fetchJson<void>(`/downloads/${id}/cancel`, { method: 'POST' }),
  
  getDownloadStats: () => fetchJson<DownloadStats>('/downloads/stats'),
  
  // Settings
  getSettings: () => fetchJson<Settings>('/settings'),
  
  updateSettings: (data: Partial<Settings>) =>
    fetchJson<Settings>('/settings', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  
  // File Types
  getFileTypes: () => fetchJson<Record<string, FileTypeConfig>>('/file-types'),
  
  addFileType: (data: {
    name: string
    extensions: string[]
    destination: string
  }) =>
    fetchJson<{ id: string }>('/file-types', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  
  updateFileType: (
    id: string,
    data: Partial<{ name: string; extensions: string[]; destination: string }>
  ) =>
    fetchJson<void>(`/file-types/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  
  removeFileType: (id: string) =>
    fetchJson<void>(`/file-types/${id}`, { method: 'DELETE' }),
  
  // URL utilities
  getUrlInfo: (url: string) =>
    fetchJson<{ filename: string | null; size: number | null; content_type: string | null }>('/url-info', {
      method: 'POST',
      body: JSON.stringify({ url }),
    }),
}
