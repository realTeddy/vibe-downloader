import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Save, Plus, Trash2, Loader2, Folder } from 'lucide-react'
import toast from 'react-hot-toast'
import { api } from '../api/client'

export function SettingsPanel() {
  const queryClient = useQueryClient()

  const { data: settings, isLoading: settingsLoading } = useQuery({
    queryKey: ['settings'],
    queryFn: api.getSettings,
  })

  const { data: fileTypes, isLoading: fileTypesLoading } = useQuery({
    queryKey: ['fileTypes'],
    queryFn: api.getFileTypes,
  })

  const updateSettingsMutation = useMutation({
    mutationFn: api.updateSettings,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] })
      toast.success('Settings saved')
    },
    onError: (err: Error) => {
      toast.error(`Failed to save settings: ${err.message}`)
    },
  })

  const [maxConcurrent, setMaxConcurrent] = useState<number | null>(null)
  const [startOnLogin, setStartOnLogin] = useState<boolean | null>(null)

  const currentMaxConcurrent = maxConcurrent ?? settings?.max_concurrent_downloads ?? 3
  const currentStartOnLogin = startOnLogin ?? settings?.start_on_login ?? false

  const handleSaveSettings = () => {
    updateSettingsMutation.mutate({
      max_concurrent_downloads: currentMaxConcurrent,
      start_on_login: currentStartOnLogin,
    })
  }

  if (settingsLoading || fileTypesLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    )
  }

  return (
    <div className="space-y-4 sm:space-y-6">
      {/* General Settings */}
      <section className="bg-white dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800 p-4 sm:p-6">
        <h2 className="text-base sm:text-lg font-semibold text-slate-800 dark:text-white mb-4">
          General Settings
        </h2>

        <div className="space-y-4">
          {/* Max Concurrent Downloads */}
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              Maximum Concurrent Downloads
            </label>
            <input
              type="number"
              min={1}
              max={10}
              value={currentMaxConcurrent}
              onChange={(e) => setMaxConcurrent(parseInt(e.target.value) || 1)}
              className="w-full sm:w-32 px-3 py-3 sm:py-2 border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500 text-base"
            />
            <p className="text-xs text-slate-500 dark:text-slate-500 mt-2">
              Downloads exceeding this limit will be queued
            </p>
          </div>

          {/* Start on Login */}
          <div className="flex items-center gap-3 py-2">
            <input
              type="checkbox"
              id="startOnLogin"
              checked={currentStartOnLogin}
              onChange={(e) => setStartOnLogin(e.target.checked)}
              className="w-5 h-5 text-primary-500 rounded focus:ring-primary-500"
            />
            <label
              htmlFor="startOnLogin"
              className="text-sm font-medium text-slate-700 dark:text-slate-300"
            >
              Start on system login
            </label>
          </div>

          <button
            onClick={handleSaveSettings}
            disabled={updateSettingsMutation.isPending}
            className="w-full sm:w-auto flex items-center justify-center gap-2 px-4 py-3 sm:py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 active:bg-primary-700 transition-colors disabled:opacity-50 font-medium"
          >
            {updateSettingsMutation.isPending ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Save className="w-4 h-4" />
            )}
            Save Settings
          </button>
        </div>
      </section>

      {/* File Types */}
      <section className="bg-white dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-800 p-4 sm:p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-base sm:text-lg font-semibold text-slate-800 dark:text-white">
            File Types
          </h2>
          <AddFileTypeButton />
        </div>

        <div className="space-y-2">
          {fileTypes &&
            Object.entries(fileTypes).map(([id, config]) => (
              <FileTypeItem key={id} id={id} config={config} />
            ))}
        </div>
      </section>
    </div>
  )
}

function AddFileTypeButton() {
  const [isOpen, setIsOpen] = useState(false)
  const [name, setName] = useState('')
  const [extensions, setExtensions] = useState('')
  const [destination, setDestination] = useState('')
  
  const queryClient = useQueryClient()

  const addMutation = useMutation({
    mutationFn: api.addFileType,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['fileTypes'] })
      toast.success('File type added')
      setIsOpen(false)
      setName('')
      setExtensions('')
      setDestination('')
    },
    onError: (err: Error) => {
      toast.error(`Failed to add file type: ${err.message}`)
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    
    const extArray = extensions.split(',').map((e) => e.trim().replace(/^\./g, ''))
    
    addMutation.mutate({
      name,
      extensions: extArray,
      destination,
    })
  }

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="flex items-center gap-2 px-3 py-2 text-sm bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700 active:bg-slate-300 dark:active:bg-slate-600 transition-colors"
      >
        <Plus className="w-4 h-4" />
        <span className="hidden sm:inline">Add Type</span>
      </button>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60">
      <div className="bg-white dark:bg-slate-900 rounded-xl p-4 w-full max-w-sm border border-slate-200 dark:border-slate-800">
        <h3 className="text-lg font-semibold text-slate-800 dark:text-white mb-4">Add File Type</h3>
        <div className="space-y-3">
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Name (e.g. Images)"
            className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white"
            required
          />
          <input
            type="text"
            value={extensions}
            onChange={(e) => setExtensions(e.target.value)}
            placeholder="Extensions (e.g. jpg, png, gif)"
            className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white"
            required
          />
          <input
            type="text"
            value={destination}
            onChange={(e) => setDestination(e.target.value)}
            placeholder="Destination folder path"
            className="w-full px-3 py-3 text-base border border-slate-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-slate-800 dark:text-white"
            required
          />
        </div>
        <div className="flex gap-3 mt-4">
          <button
            type="button"
            onClick={() => setIsOpen(false)}
            className="flex-1 px-4 py-3 text-sm font-medium text-slate-600 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={addMutation.isPending}
            className="flex-1 px-4 py-3 text-sm font-medium bg-primary-500 text-white rounded-lg hover:bg-primary-600 disabled:opacity-50"
          >
            {addMutation.isPending ? 'Adding...' : 'Add'}
          </button>
        </div>
      </div>
    </form>
  )
}

interface FileTypeItemProps {
  id: string
  config: {
    name: string
    extensions: string[]
    destination: string
  }
}

function FileTypeItem({ id, config }: FileTypeItemProps) {
  const queryClient = useQueryClient()

  const removeMutation = useMutation({
    mutationFn: api.removeFileType,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['fileTypes'] })
      toast.success('File type removed')
    },
    onError: (err: Error) => {
      toast.error(`Failed to remove file type: ${err.message}`)
    },
  })

  return (
    <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <Folder className="w-4 h-4 text-slate-400 flex-shrink-0" />
          <span className="font-medium text-slate-800 dark:text-white truncate">
            {config.name}
          </span>
        </div>
        <p className="text-xs text-slate-500 dark:text-slate-500 mt-1 ml-6 truncate">
          {config.extensions.join(', ')}
        </p>
        <p className="text-xs text-slate-400 dark:text-slate-600 mt-0.5 ml-6 truncate">
          {config.destination}
        </p>
      </div>

      {id !== 'general' && (
        <button
          onClick={() => removeMutation.mutate(id)}
          disabled={removeMutation.isPending}
          className="p-2 text-slate-400 hover:text-red-500 active:bg-slate-100 dark:active:bg-slate-800 rounded-lg transition-colors disabled:opacity-50"
          title="Remove file type"
        >
          <Trash2 className="w-5 h-5" />
        </button>
      )}
    </div>
  )
}
