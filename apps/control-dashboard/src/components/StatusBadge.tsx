import { Badge } from './ui/Badge'
import type { Machine } from '../api/machines'

type ConnectivityStatus = Machine['connectivity_status']

const statusConfig: Record<ConnectivityStatus, { variant: 'online' | 'pending' | 'offline'; label: string }> = {
  online: { variant: 'online', label: 'Online' },
  stale: { variant: 'pending', label: 'Stale' },
  offline: { variant: 'offline', label: 'Offline' },
  unknown: { variant: 'offline', label: 'Unknown' },
}

export function StatusBadge({ status }: { status: ConnectivityStatus }) {
  const config = statusConfig[status]
  return <Badge variant={config.variant}>{config.label}</Badge>
}
