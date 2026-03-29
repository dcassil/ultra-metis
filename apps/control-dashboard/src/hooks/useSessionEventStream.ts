import { useState, useEffect, useRef, useCallback } from 'react'
import type { SessionOutputEvent } from '../api/events'

const API_BASE = import.meta.env.VITE_API_BASE_URL || `${window.location.protocol}//${window.location.hostname}:3000`
const AUTH_TOKEN = import.meta.env.VITE_AUTH_TOKEN || 'static-token'

const MAX_EVENTS = 500
const INITIAL_RETRY_MS = 1_000
const MAX_RETRY_MS = 30_000

export interface UseSessionEventStreamOptions {
  /** Pre-loaded historical events to seed the stream with */
  initialEvents?: SessionOutputEvent[]
  /** Only append SSE events with sequence_num greater than this value */
  startAfterSequence?: number
  /** Whether to open the SSE connection (set false for terminal sessions) */
  enabled?: boolean
}

interface UseSessionEventStreamResult {
  events: SessionOutputEvent[]
  isConnected: boolean
  error: string | null
}

export function useSessionEventStream(
  sessionId: string,
  options: UseSessionEventStreamOptions = {},
): UseSessionEventStreamResult {
  const { initialEvents, startAfterSequence, enabled = true } = options

  const [events, setEvents] = useState<SessionOutputEvent[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const retryDelay = useRef(INITIAL_RETRY_MS)
  const retryTimeout = useRef<ReturnType<typeof setTimeout> | null>(null)
  const eventSourceRef = useRef<EventSource | null>(null)
  const startAfterSequenceRef = useRef(startAfterSequence)

  // Keep the ref in sync so the SSE callback always reads the latest value
  startAfterSequenceRef.current = startAfterSequence

  // Seed events from initialEvents whenever they change
  useEffect(() => {
    if (initialEvents) {
      setEvents(initialEvents)
    }
  }, [initialEvents])

  const connect = useCallback(() => {
    // Clean up any existing connection
    if (eventSourceRef.current) {
      eventSourceRef.current.close()
      eventSourceRef.current = null
    }

    const url = `${API_BASE}/api/sessions/${sessionId}/events/stream?token=${encodeURIComponent(AUTH_TOKEN)}`
    const es = new EventSource(url)
    eventSourceRef.current = es

    es.onopen = () => {
      setIsConnected(true)
      setError(null)
      retryDelay.current = INITIAL_RETRY_MS
    }

    es.addEventListener('session_event', (msg) => {
      try {
        const event = JSON.parse(msg.data as string) as SessionOutputEvent

        // Deduplicate: skip events already covered by historical hydration
        const threshold = startAfterSequenceRef.current
        if (threshold !== undefined && event.sequence_num <= threshold) {
          return
        }

        setEvents((prev) => {
          // Extra safety: skip if this sequence_num is already present at the tail
          if (prev.length > 0 && prev[prev.length - 1].sequence_num >= event.sequence_num) {
            return prev
          }
          const next = [...prev, event]
          // Trim to max visible events
          if (next.length > MAX_EVENTS) {
            return next.slice(next.length - MAX_EVENTS)
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
  }, [sessionId])

  useEffect(() => {
    // Reset state on session change (but don't clear initialEvents — they'll be re-seeded)
    if (!initialEvents) {
      setEvents([])
    }
    setIsConnected(false)
    setError(null)
    retryDelay.current = INITIAL_RETRY_MS

    if (enabled) {
      connect()
    }

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
  }, [connect, enabled, initialEvents])

  return { events, isConnected, error }
}
