import { useState, useEffect } from 'react'

function formatRelativeTime(isoTimestamp: string): string {
  const diffMs = Date.now() - Date.parse(isoTimestamp)
  const seconds = Math.floor(diffMs / 1000)

  if (seconds < 60) return `${seconds}s ago`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

export function RelativeTime({ timestamp }: { timestamp: string | null }) {
  const [, setTick] = useState(0)

  useEffect(() => {
    if (!timestamp) return
    const interval = setInterval(() => setTick((t) => t + 1), 10_000)
    return () => clearInterval(interval)
  }, [timestamp])

  if (!timestamp) {
    return <span className="text-secondary-400">Never</span>
  }

  return <span title={timestamp}>{formatRelativeTime(timestamp)}</span>
}
