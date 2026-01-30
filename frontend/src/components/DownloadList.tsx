import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { 
  Trash2, 
  XCircle, 
  CheckCircle, 
  Clock, 
  AlertCircle,
  Download,
  Loader2,
  Pause
} from 'lucide-react'
import toast from 'react-hot-toast'
import { api } from '../api/client'
import type { DownloadRecord, DownloadStatus } from '../types'

export function DownloadList() {
  const queryClient = useQueryClient()
  
  const { data: downloads, isLoading, error } = useQuery({
    queryKey: ['downloads'],
    queryFn: api.getDownloads,
    refetchInterval: 10000, // Refresh every 10s as backup
  })

  const removeMutation = useMutation({
    mutationFn: api.removeDownload,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['downloads'] })
      queryClient.invalidateQueries({ queryKey: ['downloadStats'] })
      toast.success('Download removed')
    },
    onError: (err: Error) => {
      toast.error(`Failed to remove: ${err.message}`)
    },
  })

  const cancelMutation = useMutation({
    mutationFn: api.cancelDownload,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['downloads'] })
      queryClient.invalidateQueries({ queryKey: ['downloadStats'] })
      toast.success('Download cancelled')
    },
    onError: (err: Error) => {
      toast.error(`Failed to cancel: ${err.message}`)
    },
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center py-12">
        <AlertCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
        <p className="text-slate-600 dark:text-slate-400">
          Failed to load downloads. Is the backend running?
        </p>
      </div>
    )
  }

  if (!downloads || downloads.length === 0) {
    return (
      <div className="text-center py-12">
        <Download className="w-12 h-12 text-slate-300 dark:text-slate-700 mx-auto mb-4" />
        <p className="text-slate-600 dark:text-slate-400">
          No downloads yet. Tap "+" to get started.
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-2 sm:space-y-3">
      {downloads.map((download) => (
        <DownloadItem
          key={download.id}
          download={download}
          onRemove={() => removeMutation.mutate(download.id)}
          onCancel={() => cancelMutation.mutate(download.id)}
        />
      ))}
    </div>
  )
}

interface DownloadItemProps {
  download: DownloadRecord
  onRemove: () => void
  onCancel: () => void
}

function DownloadItem({ download, onRemove, onCancel }: DownloadItemProps) {
  const progress = download.total_size
    ? (download.downloaded_size / download.total_size) * 100
    : 0

  const statusConfig = getStatusConfig(download.status)

  return (
    <div className="bg-white dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800 p-3 sm:p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <statusConfig.icon className={`w-4 h-4 flex-shrink-0 ${statusConfig.color}`} />
            <h3 className="font-medium text-slate-800 dark:text-white truncate text-sm sm:text-base">
              {download.filename}
            </h3>
          </div>
          
          <p className="text-xs sm:text-sm text-slate-500 dark:text-slate-500 truncate mt-1">
            {download.url}
          </p>
          
          <div className="flex flex-wrap items-center gap-2 sm:gap-4 mt-2 text-xs text-slate-500 dark:text-slate-500">
            <span className="capitalize bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded">{download.file_type}</span>
            <span>{formatBytes(download.downloaded_size)}{download.total_size ? ` / ${formatBytes(download.total_size)}` : ''}</span>
            <span className={statusConfig.color}>{statusConfig.label}</span>
          </div>
          
          {/* Progress bar */}
          {(download.status === 'downloading' || download.status === 'queued') && (
            <div className="mt-3">
              <div className="h-1.5 sm:h-2 bg-slate-200 dark:bg-slate-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary-500 transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          )}
          
          {/* Error message */}
          {download.error_message && (
            <p className="text-xs sm:text-sm text-red-500 mt-2">{download.error_message}</p>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1">
          {(download.status === 'downloading' || download.status === 'queued' || download.status === 'pending') && (
            <button
              onClick={onCancel}
              className="p-2 text-slate-400 hover:text-orange-500 active:bg-slate-100 dark:active:bg-slate-800 rounded-lg transition-colors"
              title="Cancel download"
            >
              <XCircle className="w-5 h-5" />
            </button>
          )}
          
          <button
            onClick={onRemove}
            className="p-2 text-slate-400 hover:text-red-500 active:bg-slate-100 dark:active:bg-slate-800 rounded-lg transition-colors"
            title="Remove download"
          >
            <Trash2 className="w-5 h-5" />
          </button>
        </div>
      </div>
    </div>
  )
}

function getStatusConfig(status: DownloadStatus) {
  switch (status) {
    case 'pending':
      return { icon: Clock, color: 'text-slate-400', label: 'Pending' }
    case 'queued':
      return { icon: Clock, color: 'text-yellow-500', label: 'Queued' }
    case 'downloading':
      return { icon: Loader2, color: 'text-blue-500', label: 'Downloading' }
    case 'paused':
      return { icon: Pause, color: 'text-yellow-500', label: 'Paused' }
    case 'completed':
      return { icon: CheckCircle, color: 'text-green-500', label: 'Completed' }
    case 'failed':
      return { icon: AlertCircle, color: 'text-red-500', label: 'Failed' }
    case 'cancelled':
      return { icon: XCircle, color: 'text-orange-500', label: 'Cancelled' }
    default:
      return { icon: Clock, color: 'text-slate-400', label: status }
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
}
