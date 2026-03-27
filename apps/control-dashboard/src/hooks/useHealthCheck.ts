import { useState, useEffect, useCallback } from 'react'
import { checkHealth } from '../api/health'

export type ConnectionStatus = 'connected' | 'degraded' | 'disconnected'

interface HealthCheckResult {
  status: ConnectionStatus
  lastChecked: Date | null
  error: string | null
}

const POLL_INTERVAL_MS = 30_000

export function useHealthCheck(): HealthCheckResult {
  const [status, setStatus] = useState<ConnectionStatus>('disconnected')
  const [lastChecked, setLastChecked] = useState<Date | null>(null)
  const [error, setError] = useState<string | null>(null)

  const check = useCallback(async () => {
    const result = await checkHealth()
    setLastChecked(new Date())

    if (result.status === 'ok') {
      setStatus('connected')
      setError(null)
    } else if (result.status === 'degraded') {
      setStatus('degraded')
      setError(null)
    } else {
      setStatus('disconnected')
      setError('Unable to reach the Control Service')
    }
  }, [])

  useEffect(() => {
    check()
    const interval = setInterval(check, POLL_INTERVAL_MS)
    return () => clearInterval(interval)
  }, [check])

  return { status, lastChecked, error }
}
