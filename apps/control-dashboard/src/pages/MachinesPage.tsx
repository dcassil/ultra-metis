import { useState, useEffect, useCallback, useRef, useMemo } from 'react'
import type { Machine } from '../api/machines'
import { listMachines } from '../api/machines'
import { Table, Badge } from '../components/ui'
import PendingMachinesBanner from '../components/PendingMachinesBanner'
import PendingMachineCard from '../components/PendingMachineCard'

type StatusVariant = 'online' | 'offline' | 'pending' | 'error'
const statusVariant: Record<string, StatusVariant> = {
  online: 'online',
  stale: 'pending',
  offline: 'offline',
  pending: 'pending',
}

type TrustVariant = 'online' | 'pending' | 'error'
const trustVariant: Record<string, TrustVariant> = {
  trusted: 'online',
  restricted: 'pending',
  pending: 'error',
}

const columns = [
  { key: 'name', header: 'Name' },
  {
    key: 'platform',
    header: 'Platform',
  },
  {
    key: 'status',
    header: 'Status',
    render: (row: Record<string, unknown>) => (
      <Badge variant={statusVariant[row.status as string] ?? 'offline'}>
        {String(row.status)}
      </Badge>
    ),
  },
  {
    key: 'trust_tier',
    header: 'Trust Tier',
    render: (row: Record<string, unknown>) => (
      <Badge variant={trustVariant[row.trust_tier as string] ?? 'offline'}>
        {String(row.trust_tier)}
      </Badge>
    ),
  },
  {
    key: 'repos_count',
    header: 'Repos',
    render: (row: Record<string, unknown>) => {
      const count = row.repos_count as number
      return count > 0 ? String(count) : '\u2014'
    },
  },
]

export default function MachinesPage() {
  const [machines, setMachines] = useState<Machine[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showPending, setShowPending] = useState(false)
  const pendingSectionRef = useRef<HTMLDivElement>(null)

  const fetchMachines = useCallback(async () => {
    try {
      const data = await listMachines()
      setMachines(data)
      setError(null)
    } catch {
      setError('Failed to load machines')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchMachines()
  }, [fetchMachines])

  const pendingMachines = useMemo(
    () => machines.filter((m) => m.status === 'pending'),
    [machines],
  )

  const activeMachines = useMemo(
    () => machines.filter((m) => m.status !== 'pending') as (Machine & Record<string, unknown>)[],
    [machines],
  )

  function handleViewPending() {
    setShowPending(true)
    // Scroll to the pending section after render
    setTimeout(() => {
      pendingSectionRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }, 50)
  }

  function handlePendingAction() {
    fetchMachines()
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="text-sm text-secondary-500">Loading machines...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="rounded-lg border border-danger-200 bg-danger-50 px-8 py-12 text-center">
          <p className="text-sm text-danger-700">{error}</p>
          <button
            type="button"
            onClick={() => {
              setLoading(true)
              fetchMachines()
            }}
            className="mt-3 text-sm font-medium text-primary-600 hover:text-primary-700"
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-semibold text-secondary-900">Machines</h2>
      </div>

      <PendingMachinesBanner count={pendingMachines.length} onViewPending={handleViewPending} />

      {showPending && pendingMachines.length > 0 && (
        <div ref={pendingSectionRef} className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium text-secondary-900">Pending Approval</h3>
            <button
              type="button"
              onClick={() => setShowPending(false)}
              className="text-sm text-secondary-500 hover:text-secondary-700"
            >
              Hide
            </button>
          </div>
          {pendingMachines.map((machine) => (
            <PendingMachineCard
              key={machine.id}
              machine={machine}
              onAction={handlePendingAction}
            />
          ))}
        </div>
      )}

      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {activeMachines.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">No machines registered yet.</p>
          </div>
        ) : (
          <Table columns={columns} data={activeMachines} />
        )}
      </div>
    </div>
  )
}
