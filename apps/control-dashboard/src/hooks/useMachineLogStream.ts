import { useState, useEffect, useRef, useCallback } from 'react'
import type { MachineLogEntry } from '../api/machineLogs'

const API_BASE = import.meta.env.VITE_API_BASE_URL || `${window.location.protocol}//${window.location.hostname}:3000`
const AUTH_TOKEN = import.meta.env.VITE_AUTH_TOKEN || 'static-token'

const MAX_ENTRIES = 500
const INITIAL_RETRY_MS = 1_000
const MAX_RETRY_MS = 30_000

interface UseMachineLogStreamResult {
  logs: MachineLogEntry[]
  isConnected: boolean
  error: string | null
}

export function useMachineLogStream(machineId: string): UseMachineLogStreamResult {
  const [logs, setLogs] = useState<MachineLogEntry[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const retryDelay = useRef(INITIAL_RETRY_MS)
  const retryTimeout = useRef<ReturnType<typeof setTimeout> | null>(null)
  const eventSourceRef = useRef<EventSource | null>(null)

  const connect = useCallback(() => {
    // Clean up any existing connection
    if (eventSourceRef.current) {
      eventSourceRef.current.close()
      eventSourceRef.current = null
    }

    const url = `${API_BASE}/api/machines/${machineId}/logs/stream?token=${encodeURIComponent(AUTH_TOKEN)}`
    const es = new EventSource(url)
    eventSourceRef.current = es

    es.onopen = () => {
      setIsConnected(true)
      setError(null)
      retryDelay.current = INITIAL_RETRY_MS
    }

    es.addEventListener('log_entry', (msg) => {
      try {
        const entry = JSON.parse(msg.data as string) as MachineLogEntry
        setLogs((prev) => {
          const next = [...prev, entry]
          if (next.length > MAX_ENTRIES) {
            return next.slice(next.length - MAX_ENTRIES)
          }
          return next
        })
      } catch {
        // Ignore unparseable messages
      }
    })

    es.onerror = () => {
      setIsConnected(false)
      es.close()
      eventSourceRef.current = null

      // Exponential backoff reconnect
      const delay = retryDelay.current
      setError(`Disconnected. Reconnecting in ${Math.round(delay / 1000)}s...`)

      retryTimeout.current = setTimeout(() => {
        retryDelay.current = Math.min(retryDelay.current * 2, MAX_RETRY_MS)
        connect()
      }, delay)
    }
  }, [machineId])

  useEffect(() => {
    // Reset state on machine change
    setLogs([])
    setIsConnected(false)
    setError(null)
    retryDelay.current = INITIAL_RETRY_MS

    connect()

    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close()
        eventSourceRef.current = null
      }
      if (retryTimeout.current) {
        clearTimeout(retryTimeout.current)
        retryTimeout.current = null
      }
    }
  }, [connect])

  return { logs, isConnected, error }
}
