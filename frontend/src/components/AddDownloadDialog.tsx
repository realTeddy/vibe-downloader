import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { X, Loader2 } from 'lucide-react'
import toast from 'react-hot-toast'
import { api } from '../api/client'

interface AddDownloadDialogProps {
  onClose: () => void
}

export function AddDownloadDialog({ onClose }: AddDownloadDialogProps) {
  const [url, setUrl] = useState('')
  const [fileType, setFileType] = useState('general')
  const [filename, setFilename] = useState('')
  
  const queryClient = useQueryClient()

  const { data: fileTypes } = useQuery({
    queryKey: ['fileTypes'],
    queryFn: api.getFileTypes,
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
    },
  })

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
            <input
              type="url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://example.com/file.zip"
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
              Filename (optional)
            </label>
            <input
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              placeholder="Leave empty to auto-detect"
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
              Add Download
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
