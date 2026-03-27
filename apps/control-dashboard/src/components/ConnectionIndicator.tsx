import { useHealth } from '../contexts/HealthContext'
import type { ConnectionStatus } from '../hooks/useHealthCheck'

const statusConfig: Record<ConnectionStatus, { dot: string; label: string }> = {
  connected: { dot: 'bg-success-500', label: 'Connected' },
  degraded: { dot: 'bg-warning-500', label: 'Degraded' },
  disconnected: { dot: 'bg-danger-500', label: 'Disconnected' },
}

export default function ConnectionIndicator() {
  const { status, lastChecked } = useHealth()
  const config = statusConfig[status]
  const title = lastChecked
    ? `Last checked: ${lastChecked.toLocaleTimeString()}`
    : 'Checking...'

  return (
    <div className="flex items-center gap-2" title={title}>
      <span className={`h-2 w-2 rounded-full ${config.dot}`} />
      <span className="text-xs text-secondary-500">{config.label}</span>
    </div>
  )
}
