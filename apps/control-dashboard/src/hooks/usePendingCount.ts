import { useState, useEffect } from 'react'
import { listMachines } from '../api/machines'

const POLL_INTERVAL_MS = 30_000

export function usePendingCount(): number {
  const [count, setCount] = useState(0)

  useEffect(() => {
    let cancelled = false

    async function fetch() {
      try {
        const machines = await listMachines()
        if (!cancelled) {
          setCount(machines.filter((m) => m.status === 'pending').length)
        }
      } catch {
        // Silently ignore — the health check will surface connectivity issues
      }
    }

    fetch()
    const interval = setInterval(fetch, POLL_INTERVAL_MS)
    return () => {
      cancelled = true
      clearInterval(interval)
    }
  }, [])

  return count
}
