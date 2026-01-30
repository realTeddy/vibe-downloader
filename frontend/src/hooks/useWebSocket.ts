import { useEffect, useRef, useCallback } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import type { ProgressUpdate } from '../types'

export function useWebSocket() {
  const wsRef = useRef<WebSocket | null>(null)
  const queryClient = useQueryClient()
  const reconnectTimeoutRef = useRef<number>()

  const connect = useCallback(() => {
    // Determine WebSocket URL based on current location
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = window.location.host
    const wsUrl = `${protocol}//${host}/ws`

    const ws = new WebSocket(wsUrl)
    wsRef.current = ws

    ws.onopen = () => {
      console.log('WebSocket connected')
    }

    ws.onmessage = (event) => {
      try {
        const update: ProgressUpdate = JSON.parse(event.data)
        
        // Update the downloads cache with new progress
        queryClient.setQueryData<any[]>(['downloads'], (oldData) => {
          if (!oldData) return oldData
          
          return oldData.map((download) => {
            if (download.id === update.id) {
              return {
                ...download,
                downloaded_size: update.downloaded,
                total_size: update.total,
                status: update.status,
                error_message: update.error,
              }
            }
            return download
          })
        })
        
        // Also invalidate stats query
        queryClient.invalidateQueries({ queryKey: ['downloadStats'] })
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e)
      }
    }

    ws.onclose = () => {
      console.log('WebSocket disconnected, reconnecting in 3s...')
      reconnectTimeoutRef.current = window.setTimeout(connect, 3000)
    }

    ws.onerror = (error) => {
      console.error('WebSocket error:', error)
      ws.close()
    }
  }, [queryClient])

  useEffect(() => {
    connect()

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [connect])

  return wsRef.current
}
