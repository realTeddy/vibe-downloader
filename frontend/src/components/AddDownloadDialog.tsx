import { useState, useEffect, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { X, Loader2 } from 'lucide-react'
import toast from 'react-hot-toast'
import { api } from '../api/client'

interface AddDownloadDialogProps {
  onClose: () => void
}

// Local storage keys for remembering selections
const LAST_FILE_TYPE_KEY = 'vibe-downloader-last-file-type'
const FILE_TYPE_HISTORY_KEY = 'vibe-downloader-file-type-history'

/** Get remembered file type for an extension */
function getRememberedFileType(filename: string): string | null {
  try {
    const ext = filename.split('.').pop()?.toLowerCase()
    if (!ext) return null
    
    const history = JSON.parse(localStorage.getItem(FILE_TYPE_HISTORY_KEY) || '{}')
    return history[ext] || null
  } catch {
    return null
  }
}

/** Remember file type selection for an extension */
function rememberFileTypeForExtension(filename: string, fileType: string) {
  try {
    const ext = filename.split('.').pop()?.toLowerCase()
    if (!ext) return
    
    const history = JSON.parse(localStorage.getItem(FILE_TYPE_HISTORY_KEY) || '{}')
    history[ext] = fileType
    localStorage.setItem(FILE_TYPE_HISTORY_KEY, JSON.stringify(history))
  } catch {
    // Ignore storage errors
  }
}

/** Auto-detect file type based on filename extension */
function detectFileTypeFromFilename(
  filename: string,
  fileTypes: Record<string, { name: string; extensions: string[]; destination: string }> | undefined
): string {
  if (!fileTypes || !filename) return 'general'
  
  // First check user's history for this extension
  const remembered = getRememberedFileType(filename)
  if (remembered && fileTypes[remembered]) {
    return remembered
  }
  
  // Fall back to extension matching
  const ext = filename.split('.').pop()?.toLowerCase()
  if (ext) {
    for (const [id, config] of Object.entries(fileTypes)) {
      if (config.extensions.some(e => e.toLowerCase() === ext)) {
        return id
      }
    }
  }
  return 'general'
}

/** Get last used file type */
function getLastFileType(): string {
  try {
    return localStorage.getItem(LAST_FILE_TYPE_KEY) || 'general'
  } catch {
    return 'general'
  }
}

/** Save last used file type */
function saveLastFileType(fileType: string) {
  try {
    localStorage.setItem(LAST_FILE_TYPE_KEY, fileType)
  } catch {
    // Ignore storage errors
  }
}

export function AddDownloadDialog({ onClose }: AddDownloadDialogProps) {
  const [url, setUrl] = useState('')
  const [fileType, setFileType] = useState(() => getLastFileType())
  const [filename, setFilename] = useState('')
  const [fetchingInfo, setFetchingInfo] = useState(false)
  const lastFetchedUrl = useRef<string>('')
  
  const queryClient = useQueryClient()

  const { data: fileTypes } = useQuery({
    queryKey: ['fileTypes'],
    queryFn: api.getFileTypes,
  })

  const addMutation = useMutation({
    mutationFn: api.addDownload,
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['downloads'] })
      
      // Remember selections
      saveLastFileType(fileType)
      if (filename) {
        rememberFileTypeForExtension(filename, fileType)
      }
      
      toast.success(data.queued ? 'Download queued' : 'Download started')
      onClose()
    },
    onError: (err: Error) => {
      toast.error(`Failed to add download: ${err.message}`)
    },
  })
  
  // Fetch URL info when URL changes
  useEffect(() => {
    const trimmedUrl = url.trim()
    if (
      trimmedUrl &&
      (trimmedUrl.startsWith('http://') || trimmedUrl.startsWith('https://')) &&
      trimmedUrl !== lastFetchedUrl.current
    ) {
      lastFetchedUrl.current = trimmedUrl
      setFetchingInfo(true)
      
      api.getUrlInfo(trimmedUrl)
        .then((info) => {
          if (info.filename) {
            setFilename(info.filename)
            // Auto-detect file type from the fetched filename
            const detected = detectFileTypeFromFilename(info.filename, fileTypes)
            setFileType(detected)
          }
        })
        .catch(() => {
          // Silently fail - user can still enter filename manually
        })
        .finally(() => {
          setFetchingInfo(false)
        })
    }
  }, [url, fileTypes])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!url.trim()) {
      toast.error('Please enter a URL')
      return
    }
    
    addMutation.mutate({
      url: url.trim(),
      file_type: fileType,
      filename: filename.trim() || undefined,
    })
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-end sm:items-center justify-center z-50">
      <div className="bg-white dark:bg-slate-900 rounded-t-2xl sm:rounded-xl shadow-xl w-full sm:max-w-lg sm:mx-4 border-t sm:border border-slate-200 dark:border-slate-800">
        <div className="flex items-center justify-between p-4 border-b border-slate-200 dark:border-slate-800">
          <h2 className="text-lg font-semibold text-slate-800 dark:text-white">
            Add Download
          </h2>
          <button
            onClick={onClose}
            className="p-2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 active:bg-slate-100 dark:active:bg-slate-800 rounded-lg"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          {/* URL Input */}
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              URL
            </label>
            <div className="relative">
              <input
                type="url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://example.com/file.zip"
                className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
                autoFocus
              />
              {fetchingInfo && (
                <div className="absolute right-3 top-1/2 -translate-y-1/2">
                  <Loader2 className="w-4 h-4 animate-spin text-slate-400" />
                </div>
              )}
            </div>
          </div>

          {/* Filename */}
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              Filename
            </label>
            <input
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              placeholder={fetchingInfo ? "Detecting..." : "Auto-detected from server"}
              className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>

          {/* File Type / Destination Select */}
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              Save to
            </label>
            <select
              value={fileType}
              onChange={(e) => setFileType(e.target.value)}
              className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              {fileTypes &&
                Object.entries(fileTypes).map(([id, config]) => (
                  <option key={id} value={id}>
                    {config.name} — {config.destination}
                  </option>
                ))}
            </select>
          </div>

          {/* Actions */}
          <div className="flex flex-col-reverse sm:flex-row justify-end gap-3 pt-2 pb-4 sm:pb-0">
            <button
              type="button"
              onClick={onClose}
              className="w-full sm:w-auto px-4 py-3 sm:py-2 text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 dark:hover:bg-slate-700 rounded-lg transition-colors font-medium"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={addMutation.isPending || !url.trim()}
              className="w-full sm:w-auto flex items-center justify-center gap-2 px-4 py-3 sm:py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 active:bg-primary-700 transition-colors disabled:opacity-50 font-medium"
            >
              {addMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
              {addMutation.isPending ? 'Starting...' : 'Download'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
