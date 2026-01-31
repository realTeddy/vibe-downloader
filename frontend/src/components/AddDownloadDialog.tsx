import { useState, useEffect, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { X, Loader2, Zap } from 'lucide-react'
import toast from 'react-hot-toast'
import { api } from '../api/client'

interface AddDownloadDialogProps {
  onClose: () => void
}

/** Extract filename from URL */
function extractFilenameFromUrl(urlString: string): string | null {
  try {
    const url = new URL(urlString)
    const pathname = url.pathname
    // Get the last segment of the path
    const segments = pathname.split('/').filter(Boolean)
    if (segments.length > 0) {
      const lastSegment = segments[segments.length - 1]
      // Decode URI and check if it looks like a filename (has extension)
      const decoded = decodeURIComponent(lastSegment)
      if (decoded.includes('.') && !decoded.startsWith('.')) {
        return decoded
      }
    }
  } catch {
    // Invalid URL
  }
  return null
}

/** Auto-detect file type based on URL extension */
function detectFileType(
  urlString: string,
  fileTypes: Record<string, { name: string; extensions: string[]; destination: string }> | undefined
): string {
  if (!fileTypes) return 'general'
  
  try {
    const url = new URL(urlString)
    const pathname = url.pathname.toLowerCase()
    const ext = pathname.split('.').pop()
    
    if (ext) {
      for (const [id, config] of Object.entries(fileTypes)) {
        if (config.extensions.some(e => e.toLowerCase() === ext.toLowerCase())) {
          return id
        }
      }
    }
  } catch {
    // Invalid URL
  }
  return 'general'
}

export function AddDownloadDialog({ onClose }: AddDownloadDialogProps) {
  const [url, setUrl] = useState('')
  const [fileType, setFileType] = useState('general')
  const [filename, setFilename] = useState('')
  const [autoStarted, setAutoStarted] = useState(false)
  const urlInputRef = useRef<HTMLInputElement>(null)
  
  const queryClient = useQueryClient()

  const { data: fileTypes } = useQuery({
    queryKey: ['fileTypes'],
    queryFn: api.getFileTypes,
  })
  
  const { data: settings } = useQuery({
    queryKey: ['settings'],
    queryFn: api.getSettings,
  })
  
  const { data: downloads } = useQuery({
    queryKey: ['downloads'],
    queryFn: api.getDownloads,
  })

  const addMutation = useMutation({
    mutationFn: api.addDownload,
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['downloads'] })
      queryClient.invalidateQueries({ queryKey: ['downloadStats'] })
      toast.success(data.queued ? 'Download queued' : 'Download started')
      onClose()
    },
    onError: (err: Error) => {
      toast.error(`Failed to add download: ${err.message}`)
      setAutoStarted(false) // Allow retry
    },
  })
  
  // Check if we have capacity for auto-start
  const activeDownloads = downloads?.filter(d => d.status === 'downloading').length ?? 0
  const maxConcurrent = settings?.max_concurrent_downloads ?? 3
  const hasCapacity = activeDownloads < maxConcurrent
  
  // Handle URL change - extract filename and auto-detect file type
  const handleUrlChange = (newUrl: string) => {
    setUrl(newUrl)
    
    // Only process if it looks like a valid URL
    if (newUrl.startsWith('http://') || newUrl.startsWith('https://')) {
      // Extract filename
      const extracted = extractFilenameFromUrl(newUrl)
      if (extracted && !filename) {
        setFilename(extracted)
      }
      
      // Auto-detect file type
      const detected = detectFileType(newUrl, fileTypes)
      setFileType(detected)
    }
  }
  
  // Auto-start download when URL is pasted and there's capacity
  useEffect(() => {
    if (
      url &&
      hasCapacity &&
      !autoStarted &&
      !addMutation.isPending &&
      (url.startsWith('http://') || url.startsWith('https://'))
    ) {
      // Small delay to allow user to see what's happening
      const timer = setTimeout(() => {
        setAutoStarted(true)
        addMutation.mutate({
          url: url.trim(),
          file_type: fileType,
          filename: filename.trim() || undefined,
        })
      }, 300)
      
      return () => clearTimeout(timer)
    }
  }, [url, hasCapacity, autoStarted, addMutation.isPending, fileType, filename])

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
          <div className="flex items-center gap-2">
            <h2 className="text-lg font-semibold text-slate-800 dark:text-white">
              Add Download
            </h2>
            {hasCapacity && (
              <span className="flex items-center gap-1 text-xs px-2 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 rounded-full">
                <Zap className="w-3 h-3" />
                Auto-start
              </span>
            )}
          </div>
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
            <input
              ref={urlInputRef}
              type="url"
              value={url}
              onChange={(e) => handleUrlChange(e.target.value)}
              placeholder="Paste URL to auto-start download"
              className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
              autoFocus
            />
          </div>

          {/* File Type Select */}
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              File Type
            </label>
            <select
              value={fileType}
              onChange={(e) => setFileType(e.target.value)}
              className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              {fileTypes &&
                Object.entries(fileTypes).map(([id, config]) => (
                  <option key={id} value={id}>
                    {config.name}
                  </option>
                ))}
            </select>
            {fileTypes && fileTypes[fileType] && (
              <p className="text-xs text-slate-500 dark:text-slate-500 mt-2">
                Saves to: {fileTypes[fileType].destination}
              </p>
            )}
          </div>

          {/* Custom Filename (optional) */}
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              Filename
            </label>
            <input
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              placeholder="Auto-detected from URL"
              className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
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
              disabled={addMutation.isPending}
              className="w-full sm:w-auto flex items-center justify-center gap-2 px-4 py-3 sm:py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 active:bg-primary-700 transition-colors disabled:opacity-50 font-medium"
            >
              {addMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
              {addMutation.isPending ? 'Starting...' : 'Add Download'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
