import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { listMachines } from '../api/machines'
import type { Machine } from '../api/machines'
import { Table } from '../components/ui/Table'
import { Button } from '../components/ui/Button'
import { StatusBadge } from '../components/StatusBadge'
import { TrustTierBadge } from '../components/TrustTierBadge'
import { RelativeTime } from '../components/RelativeTime'

const connectivityOrder: Record<Machine['connectivity_status'], number> = {
  online: 0,
  stale: 1,
  offline: 2,
  unknown: 3,
}

function sortByConnectivity(machines: Machine[]): Machine[] {
  return [...machines].sort((a, b) => connectivityOrder[a.connectivity_status] - connectivityOrder[b.connectivity_status])
}

const columns = [
  {
    key: 'name',
    header: 'Name',
    render: (row: Machine) => <span className="font-medium text-primary-700">{row.name}</span>,
  },
  { key: 'platform', header: 'Platform' },
  {
    key: 'connectivity_status',
    header: 'Status',
    render: (row: Machine) => <StatusBadge status={row.connectivity_status} />,
  },
  {
    key: 'trust_tier',
    header: 'Trust Tier',
    render: (row: Machine) => <TrustTierBadge tier={row.trust_tier} />,
  },
  {
    key: 'repos_count',
    header: 'Repos',
    render: (row: Machine) => String(row.repos_count),
  },
  {
    key: 'last_heartbeat',
    header: 'Last Heartbeat',
    render: (row: Machine) => <RelativeTime timestamp={row.last_heartbeat} />,
  },
]

export default function MachinesPage() {
  const navigate = useNavigate()
  const [machines, setMachines] = useState<Machine[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchMachines = useCallback(async () => {
    try {
      const data = await listMachines()
      setMachines(sortByConnectivity(data))
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load machines')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void fetchMachines()
    const interval = setInterval(() => void fetchMachines(), 10_000)
    return () => clearInterval(interval)
  }, [fetchMachines])

  if (loading) {
    return (
      <div className="flex items-center justify-center py-24">
        <svg className="h-8 w-8 animate-spin text-primary-600" viewBox="0 0 24 24" fill="none">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center py-24 gap-4">
        <p className="text-danger-600">{error}</p>
        <Button variant="secondary" onClick={() => { setLoading(true); void fetchMachines() }}>
          Retry
        </Button>
      </div>
    )
  }

  if (machines.length === 0) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="rounded-lg border border-secondary-200 bg-white px-8 py-12 text-center shadow-sm">
          <h2 className="text-2xl font-semibold text-secondary-900">No Machines</h2>
          <p className="mt-2 text-secondary-500">
            No machines registered. Install the Machine Runner to get started.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold text-secondary-900">Machines</h1>
        <span className="text-sm text-secondary-500">{machines.length} registered</span>
      </div>
      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        <Table<Machine>
          columns={columns}
          data={machines}
          onRowClick={(row) => navigate(`/machines/${row.id}`)}
        />
      </div>
    </div>
  )
}
