import { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { getMachine, revokeMachine } from '../api/machines'
import type { MachineDetail } from '../api/machines'
import { Card } from '../components/ui/Card'
import { Table } from '../components/ui/Table'
import { Button } from '../components/ui/Button'
import { Modal } from '../components/ui/Modal'
import { Badge } from '../components/ui/Badge'
import { StatusBadge } from '../components/StatusBadge'
import { TrustTierBadge } from '../components/TrustTierBadge'
import { RelativeTime } from '../components/RelativeTime'

type Repo = MachineDetail['repos'][number]

const repoColumns = [
  {
    key: 'repo_name',
    header: 'Name',
    render: (row: Repo) => <span className="font-medium">{row.repo_name}</span>,
  },
  { key: 'repo_path', header: 'Path' },
  {
    key: 'cadre_managed',
    header: 'Cadre Managed',
    render: (row: Repo) =>
      row.cadre_managed ? (
        <Badge variant="online">Managed</Badge>
      ) : (
        <Badge variant="offline">Unmanaged</Badge>
      ),
  },
]

export default function MachineDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [machine, setMachine] = useState<MachineDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showRevokeModal, setShowRevokeModal] = useState(false)
  const [revoking, setRevoking] = useState(false)
  const [metadataExpanded, setMetadataExpanded] = useState(false)

  const fetchMachine = useCallback(async () => {
    if (!id) return
    try {
      const data = await getMachine(id)
      setMachine(data)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load machine')
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    void fetchMachine()
  }, [fetchMachine])

  const handleRevoke = async () => {
    if (!id) return
    setRevoking(true)
    try {
      await revokeMachine(id)
      navigate('/machines')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke machine')
      setShowRevokeModal(false)
    } finally {
      setRevoking(false)
    }
  }

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

  if (error && !machine) {
    return (
      <div className="flex flex-col items-center justify-center py-24 gap-4">
        <p className="text-danger-600">{error}</p>
        <Button variant="secondary" onClick={() => { setLoading(true); void fetchMachine() }}>
          Retry
        </Button>
      </div>
    )
  }

  if (!machine) return null

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to="/machines" className="text-sm text-primary-600 hover:text-primary-800">
          &larr; Back to Machines
        </Link>
      </div>

      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold text-secondary-900">{machine.name}</h1>
        <Button variant="danger" size="sm" onClick={() => setShowRevokeModal(true)}>
          Revoke
        </Button>
      </div>

      {error && (
        <div className="rounded-md bg-danger-50 p-3 text-sm text-danger-700">{error}</div>
      )}

      <Card title="Machine Details">
        <dl className="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-3">
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Platform</dt>
            <dd className="mt-1 text-sm text-secondary-900">{machine.platform}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Status</dt>
            <dd className="mt-1">
              <StatusBadge status={machine.connectivity_status} />
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Trust Status</dt>
            <dd className="mt-1">
              <Badge variant={machine.status === 'trusted' ? 'online' : machine.status === 'pending' ? 'pending' : 'error'}>
                {machine.status}
              </Badge>
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Trust Tier</dt>
            <dd className="mt-1">
              <TrustTierBadge tier={machine.trust_tier} />
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Last Heartbeat</dt>
            <dd className="mt-1 text-sm text-secondary-900">
              <RelativeTime timestamp={machine.last_heartbeat} />
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Created</dt>
            <dd className="mt-1 text-sm text-secondary-900">
              <RelativeTime timestamp={machine.created_at} />
            </dd>
          </div>
        </dl>
      </Card>

      <Card title="Repositories" subtitle={`${machine.repos.length} repositories`}>
        {machine.repos.length > 0 ? (
          <Table<Repo> columns={repoColumns} data={machine.repos} />
        ) : (
          <p className="text-sm text-secondary-500">No repositories linked to this machine.</p>
        )}
      </Card>

      <Card title="Metadata">
        <button
          type="button"
          className="text-sm text-primary-600 hover:text-primary-800"
          onClick={() => setMetadataExpanded(!metadataExpanded)}
        >
          {metadataExpanded ? 'Collapse' : 'Expand'} metadata
        </button>
        {metadataExpanded && (
          <pre className="mt-3 overflow-x-auto rounded-md bg-secondary-50 p-3 text-xs text-secondary-800">
            {JSON.stringify(machine.metadata, null, 2)}
          </pre>
        )}
      </Card>

      <Modal
        isOpen={showRevokeModal}
        onClose={() => setShowRevokeModal(false)}
        title="Revoke Machine"
        footer={
          <div className="flex justify-end gap-3">
            <Button variant="secondary" size="sm" onClick={() => setShowRevokeModal(false)}>
              Cancel
            </Button>
            <Button variant="danger" size="sm" loading={revoking} onClick={() => void handleRevoke()}>
              Revoke Machine
            </Button>
          </div>
        }
      >
        <p className="text-sm text-secondary-600">
          Are you sure you want to revoke <strong>{machine.name}</strong>? This machine will no longer be able to communicate with the control plane.
        </p>
      </Modal>
    </div>
  )
}
