import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Settings, Plus, Download, X, CheckCircle, Clock, Loader2 } from 'lucide-react'
import { DownloadList } from './components/DownloadList'
import { AddDownloadDialog } from './components/AddDownloadDialog'
import { SettingsPanel } from './components/SettingsPanel'
import { useWebSocket } from './hooks/useWebSocket'
import { api } from './api/client'

type Tab = 'downloads' | 'settings'

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('downloads')
  const [showAddDialog, setShowAddDialog] = useState(false)
  
  // Connect to WebSocket for real-time updates
  useWebSocket()
  
  // Fetch downloads to compute stats from actual data
  const { data: downloads } = useQuery({
    queryKey: ['downloads'],
    queryFn: api.getDownloads,
  })
  
  // Compute stats from downloads list
  const activeCount = downloads?.filter(d => d.status === 'downloading').length ?? 0
  const queuedCount = downloads?.filter(d => d.status === 'queued' || d.status === 'pending').length ?? 0
  const completedCount = downloads?.filter(d => d.status === 'completed').length ?? 0

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-black">
      {/* Header */}
      <header className="bg-white dark:bg-black border-b border-slate-200 dark:border-slate-800 sticky top-0 z-40">
        <div className="max-w-6xl mx-auto px-3 sm:px-4 py-3 sm:py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 sm:gap-3 min-w-0">
              <Download className="w-6 h-6 sm:w-8 sm:h-8 text-primary-500 flex-shrink-0" />
              <h1 className="text-lg sm:text-xl font-bold text-slate-800 dark:text-white truncate">
                {activeTab === 'settings' ? 'Settings' : 'Vibe Downloader'}
              </h1>
            </div>
            
            <nav className="flex items-center gap-2">
              {activeTab === 'settings' ? (
                <button
                  onClick={() => setActiveTab('downloads')}
                  className="flex items-center gap-2 px-3 sm:px-4 py-2 bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-200 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors"
                >
                  <X className="w-4 h-4" />
                  <span className="hidden sm:inline">Close</span>
                </button>
              ) : (
                <>
                  <button
                    onClick={() => setShowAddDialog(true)}
                    className="flex items-center gap-1 sm:gap-2 px-3 sm:px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
                  >
                    <Plus className="w-4 h-4" />
                    <span className="hidden sm:inline">Add Download</span>
                  </button>
                  
                  <button
                    onClick={() => setActiveTab('settings')}
                    className="p-2 rounded-lg text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                    title="Settings"
                  >
                    <Settings className="w-5 h-5" />
                  </button>
                </>
              )}
            </nav>
          </div>
          
          {/* Stats bar - only show on downloads tab */}
          {activeTab === 'downloads' && (
            <div className="flex items-center gap-3 sm:gap-4 mt-3 text-xs sm:text-sm overflow-x-auto pb-1">
              {activeCount > 0 && (
                <div className="flex items-center gap-1.5 text-blue-500 flex-shrink-0">
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  <span>{activeCount} active</span>
                </div>
              )}
              {queuedCount > 0 && (
                <div className="flex items-center gap-1.5 text-yellow-500 flex-shrink-0">
                  <Clock className="w-3.5 h-3.5" />
                  <span>{queuedCount} queued</span>
                </div>
              )}
              {completedCount > 0 && (
                <div className="flex items-center gap-1.5 text-green-500 flex-shrink-0">
                  <CheckCircle className="w-3.5 h-3.5" />
                  <span>{completedCount} completed</span>
                </div>
              )}
            </div>
          )}
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-6xl mx-auto px-3 sm:px-4 py-4 sm:py-6">
        {activeTab === 'downloads' ? (
          <DownloadList />
        ) : (
          <SettingsPanel />
        )}
      </main>

      {/* Add Download Dialog */}
      {showAddDialog && (
        <AddDownloadDialog onClose={() => setShowAddDialog(false)} />
      )}
    </div>
  )
}

export default App
