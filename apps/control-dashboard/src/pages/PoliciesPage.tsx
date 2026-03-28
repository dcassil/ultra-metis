import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { listMachines } from '../api/machines'
import type { Machine } from '../api/machines'
import { getMachinePolicy } from '../api/policies'
import type { MachinePolicy } from '../api/policies'
import { Table } from '../components/ui/Table'
import { SessionModeBadge } from '../components/SessionModeBadge'

interface MachineWithPolicy extends Machine {
  policy?: MachinePolicy | null
}

type MachineRow = MachineWithPolicy & Record<string, unknown>

const columns = [
  {
    key: 'name',
    header: 'Machine',
    render: (row: MachineRow) => (
      <span className="font-medium text-secondary-900">{row.name}</span>
    ),
  },
  {
    key: 'platform',
    header: 'Platform',
  },
  {
    key: 'session_mode',
    header: 'Session Mode',
    render: (row: MachineRow) => {
      const mode = row.policy?.session_mode
      return mode ? <SessionModeBadge mode={mode} /> : <span className="text-secondary-400">--</span>
    },
  },
  {
    key: 'max_autonomy_level',
    header: 'Max Autonomy',
    render: (row: MachineRow) => {
      const level = row.policy?.max_autonomy_level
      return level ? (
        <span className="text-sm text-secondary-700 capitalize">{level}</span>
      ) : (
        <span className="text-secondary-400">--</span>
      )
    },
  },
]

export default function PoliciesPage() {
  const navigate = useNavigate()
  const [machines, setMachines] = useState<MachineWithPolicy[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchData = useCallback(async () => {
    try {
      const machineList = await listMachines()
      // Fetch policies in parallel
      const withPolicies = await Promise.all(
        machineList.map(async (machine) => {
          try {
            const policy = await getMachinePolicy(machine.id)
            return { ...machine, policy }
          } catch {
            return { ...machine, policy: null }
          }
        }),
      )
      setMachines(withPolicies)
      setError(null)
    } catch {
      setError('Failed to load machines')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void fetchData()
  }, [fetchData])

  if (loading) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="text-sm text-secondary-500">Loading policies...</div>
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
              void fetchData()
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
        <h2 className="text-2xl font-semibold text-secondary-900">Policy Management</h2>
      </div>

      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {machines.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">No machines registered yet.</p>
          </div>
        ) : (
          <Table<MachineRow>
            columns={columns}
            data={machines as MachineRow[]}
            onRowClick={(row) => navigate(`/machines/${row.id}`)}
          />
        )}
      </div>
    </div>
  )
}
