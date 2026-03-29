import { useState, useEffect, useCallback, useRef, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import { CommandLineIcon } from '@heroicons/react/24/outline'
import type { Machine } from '../api/machines'
import { listMachines, deleteOfflineMachines } from '../api/machines'
import { getMachinePolicy } from '../api/policies'
import { Table, Badge, Button, Modal } from '../components/ui'
import { SessionModeBadge } from '../components/SessionModeBadge'
import { RelativeTime } from '../components/RelativeTime'
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

const connectivityDotColor: Record<string, string> = {
  online: 'bg-green-500',
  stale: 'bg-amber-400',
  offline: 'bg-secondary-400',
  unknown: 'bg-secondary-400',
}

const connectivitySortPriority: Record<string, number> = {
  online: 0,
  stale: 1,
  offline: 2,
  unknown: 3,
}

const columns = [
  {
    key: 'name',
    header: 'Name',
    render: (row: Record<string, unknown>) => {
      const connectivity = (row.connectivity_status as string) ?? 'unknown'
      const dotColor = connectivityDotColor[connectivity] ?? 'bg-secondary-400'
      return (
        <span className="flex items-center gap-2">
          <span className={`inline-block h-2 w-2 rounded-full ${dotColor}`} />
          {String(row.name)}
        </span>
      )
    },
  },
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
    key: 'session_mode',
    header: 'Session Mode',
    render: (row: Record<string, unknown>) => {
      const mode = row.session_mode as string | undefined
      return mode ? <SessionModeBadge mode={mode} /> : <span className="text-secondary-400">--</span>
    },
  },
  {
    key: 'last_heartbeat',
    header: 'Last Seen',
    render: (row: Record<string, unknown>) => (
      <RelativeTime timestamp={row.last_heartbeat as string | null} />
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
  const navigate = useNavigate()
  const [machines, setMachines] = useState<Machine[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showPending, setShowPending] = useState(false)
  const pendingSectionRef = useRef<HTMLDivElement>(null)

  const [sessionModes, setSessionModes] = useState<Record<string, string>>({})
  const [showCleanupModal, setShowCleanupModal] = useState(false)
  const [cleaning, setCleaning] = useState(false)

  const fetchMachines = useCallback(async () => {
    try {
      const data = await listMachines()
      setMachines(data)
      setError(null)
      // Fetch policies in background to get session modes
      const modes: Record<string, string> = {}
      await Promise.allSettled(
        data.map(async (m) => {
          try {
            const policy = await getMachinePolicy(m.id)
            modes[m.id] = policy.session_mode
          } catch {
            // Policy may not exist yet
          }
        }),
      )
      setSessionModes(modes)
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
    () =>
      machines
        .filter((m) => m.status !== 'pending')
        .map((m) => ({ ...m, session_mode: sessionModes[m.id] }))
        .sort(
          (a, b) =>
            (connectivitySortPriority[a.connectivity_status] ?? 3) -
            (connectivitySortPriority[b.connectivity_status] ?? 3),
        ) as (Machine & Record<string, unknown>)[],
    [machines, sessionModes],
  )

  const offlineMachineCount = useMemo(
    () => activeMachines.filter((m) => m.connectivity_status === 'offline').length,
    [activeMachines],
  )

  const tableColumns = useMemo(
    () => [
      ...columns,
      {
        key: '_actions',
        header: '',
        render: (row: Record<string, unknown>) => (
          <button
            type="button"
            title="View logs"
            onClick={(e) => {
              e.stopPropagation()
              navigate(`/machines/${row.id}?tab=logs`)
            }}
            className="rounded p-1 text-secondary-400 hover:bg-secondary-100 hover:text-secondary-600 transition-colors"
          >
            <CommandLineIcon className="h-4 w-4" />
          </button>
        ),
      },
    ],
    [navigate],
  )

  const handleCleanupOffline = async () => {
    setCleaning(true)
    try {
      await deleteOfflineMachines()
      setShowCleanupModal(false)
      setLoading(true)
      await fetchMachines()
    } catch {
      setError('Failed to remove offline machines')
    } finally {
      setCleaning(false)
    }
  }

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
        {offlineMachineCount > 0 && (
          <Button variant="ghost" size="sm" onClick={() => setShowCleanupModal(true)}>
            Remove offline ({offlineMachineCount})
          </Button>
        )}
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
          <Table
            columns={tableColumns}
            data={activeMachines}
            onRowClick={(row) => navigate(`/machines/${(row as Record<string, unknown>).id}`)}
            mobileCardConfig={{
              headerColumn: 'name',
              badgeColumn: 'status',
              bodyColumns: ['trust_tier', 'session_mode', 'last_heartbeat'],
            }}
          />
        )}
      </div>

      <Modal
        isOpen={showCleanupModal}
        onClose={() => setShowCleanupModal(false)}
        title="Remove Offline Machines"
        footer={
          <div className="flex flex-col-reverse sm:flex-row sm:justify-end gap-2 sm:gap-3">
            <Button variant="secondary" size="sm" className="w-full sm:w-auto" onClick={() => setShowCleanupModal(false)}>
              Cancel
            </Button>
            <Button variant="danger" size="sm" className="w-full sm:w-auto" loading={cleaning} onClick={() => void handleCleanupOffline()}>
              Remove Machines
            </Button>
          </div>
        }
      >
        <p className="text-sm text-secondary-600">
          Remove {offlineMachineCount} offline machine{offlineMachineCount !== 1 ? 's' : ''}? This permanently deletes all data for machines that are no longer connected.
        </p>
      </Modal>
    </div>
  )
}
