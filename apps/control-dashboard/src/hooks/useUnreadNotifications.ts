import { useState, useEffect } from 'react'
import { getUnreadCount } from '../api/notifications'

const POLL_INTERVAL_MS = 30_000

export function useUnreadNotifications(): number {
  const [count, setCount] = useState(0)

  useEffect(() => {
    let cancelled = false

    async function fetch() {
      try {
        const unread = await getUnreadCount()
        if (!cancelled) {
          setCount(unread)
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
