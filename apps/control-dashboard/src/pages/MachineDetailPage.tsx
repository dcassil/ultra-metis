import { useState, useEffect, useCallback, useMemo } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { getMachine, revokeMachine } from '../api/machines'
import type { MachineDetail } from '../api/machines'
import {
  getMachinePolicy,
  updateMachinePolicy,
  getRepoPolicy,
  updateRepoPolicy,
  listViolations,
} from '../api/policies'
import type { MachinePolicy, PolicyViolationRecord } from '../api/policies'
import { listSessions } from '../api/sessions'
import type { SessionResponse } from '../api/sessions'
import { Card } from '../components/ui/Card'
import { Table } from '../components/ui/Table'
import { Button } from '../components/ui/Button'
import { Modal } from '../components/ui/Modal'
import { Badge } from '../components/ui/Badge'
import { StatusBadge } from '../components/StatusBadge'
import { TrustTierBadge } from '../components/TrustTierBadge'
import { SessionModeBadge } from '../components/SessionModeBadge'
import { SessionStateBadge } from '../components/SessionStateBadge'
import { PolicyEditor } from '../components/PolicyEditor'
import { RelativeTime } from '../components/RelativeTime'
import { MachineLogViewer } from '../components/MachineLogViewer'

type TabId = 'details' | 'sessions' | 'logs' | 'violations'

const TABS: { id: TabId; label: string }[] = [
  { id: 'details', label: 'Details' },
  { id: 'sessions', label: 'Sessions' },
  { id: 'logs', label: 'Logs' },
  { id: 'violations', label: 'Violations' },
]

const sessionStateOptions = [
  { value: '', label: 'All States' },
  { value: 'starting', label: 'Starting' },
  { value: 'running', label: 'Running' },
  { value: 'waiting_for_input', label: 'Waiting for Input' },
  { value: 'paused', label: 'Paused' },
  { value: 'completed', label: 'Completed' },
  { value: 'failed', label: 'Failed' },
  { value: 'stopped', label: 'Stopped' },
]

function formatElapsed(startedAt: string | undefined): string {
  if (!startedAt) return '\u2014'
  const diffMs = Date.now() - Date.parse(startedAt)
  if (diffMs < 0) return '\u2014'
  const totalSeconds = Math.floor(diffMs / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  if (hours > 0) return `${hours}h ${minutes}m`
  if (minutes > 0) return `${minutes}m ${seconds}s`
  return `${seconds}s`
}

type SessionRow = SessionResponse & Record<string, unknown>
type ViolationRow = PolicyViolationRecord & Record<string, unknown>

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
  const [machinePolicy, setMachinePolicy] = useState<MachinePolicy | null>(null)
  const [policyLoading, setPolicyLoading] = useState(false)
  const [policyError, setPolicyError] = useState<string | null>(null)
  const [expandedRepoPolicy, setExpandedRepoPolicy] = useState<string | null>(null)
  const [repoPolicies, setRepoPolicies] = useState<Record<string, MachinePolicy>>({})
  const [repoPolicyLoading, setRepoPolicyLoading] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<TabId>('details')

  // Sessions tab state
  const [sessions, setSessions] = useState<SessionResponse[]>([])
  const [sessionsLoading, setSessionsLoading] = useState(false)
  const [sessionsError, setSessionsError] = useState<string | null>(null)
  const [sessionStateFilter, setSessionStateFilter] = useState('')

  // Violations tab state
  const [violations, setViolations] = useState<PolicyViolationRecord[]>([])
  const [violationsLoading, setViolationsLoading] = useState(false)
  const [violationsError, setViolationsError] = useState<string | null>(null)

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

  const fetchMachinePolicy = useCallback(async () => {
    if (!id) return
    setPolicyLoading(true)
    setPolicyError(null)
    try {
      const data = await getMachinePolicy(id)
      setMachinePolicy(data)
    } catch (err) {
      setPolicyError(err instanceof Error ? err.message : 'Failed to load policy')
    } finally {
      setPolicyLoading(false)
    }
  }, [id])

  const fetchRepoPolicy = useCallback(async (repoPath: string) => {
    if (!id) return
    setRepoPolicyLoading(repoPath)
    try {
      const data = await getRepoPolicy(id, repoPath)
      setRepoPolicies((prev) => ({ ...prev, [repoPath]: data }))
    } catch (err) {
      // If no repo-level policy exists yet, that's okay — editor will show defaults
      console.warn('Failed to load repo policy:', err)
    } finally {
      setRepoPolicyLoading(null)
    }
  }, [id])

  const fetchSessions = useCallback(async () => {
    if (!id) return
    setSessionsLoading(true)
    try {
      const params: Record<string, string> = { machine_id: id }
      if (sessionStateFilter) params.state = sessionStateFilter
      const data = await listSessions(params)
      setSessions(data.sessions)
      setSessionsError(null)
    } catch (err) {
      setSessionsError(err instanceof Error ? err.message : 'Failed to load sessions')
    } finally {
      setSessionsLoading(false)
    }
  }, [id, sessionStateFilter])

  const fetchViolations = useCallback(async () => {
    if (!id) return
    setViolationsLoading(true)
    try {
      const data = await listViolations({ machine_id: id })
      setViolations(data.violations)
      setViolationsError(null)
    } catch (err) {
      setViolationsError(err instanceof Error ? err.message : 'Failed to load violations')
    } finally {
      setViolationsLoading(false)
    }
  }, [id])

  useEffect(() => {
    void fetchMachine()
    void fetchMachinePolicy()
  }, [fetchMachine, fetchMachinePolicy])

  // Fetch sessions when tab is active or filter changes
  useEffect(() => {
    if (activeTab === 'sessions') {
      void fetchSessions()
    }
  }, [activeTab, fetchSessions])

  // Fetch violations when tab is active
  useEffect(() => {
    if (activeTab === 'violations') {
      void fetchViolations()
    }
  }, [activeTab, fetchViolations])

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

  const filteredSessions = useMemo(() => {
    const sorted = [...sessions].sort(
      (a, b) => Date.parse(b.created_at) - Date.parse(a.created_at),
    )
    return sorted as SessionRow[]
  }, [sessions])

  const sessionColumns = [
    {
      key: 'title',
      header: 'Title',
      render: (row: SessionRow) => (
        <span className="font-medium text-secondary-900">{row.title}</span>
      ),
    },
    {
      key: 'state',
      header: 'State',
      render: (row: SessionRow) => <SessionStateBadge state={row.state} />,
    },
    {
      key: 'repo_path',
      header: 'Repo',
      render: (row: SessionRow) => {
        const parts = row.repo_path.split('/')
        return <span title={row.repo_path}>{parts[parts.length - 1] || row.repo_path}</span>
      },
    },
    {
      key: 'elapsed',
      header: 'Elapsed',
      render: (row: SessionRow) => (
        <span className="text-secondary-600">{formatElapsed(row.started_at)}</span>
      ),
    },
    {
      key: 'updated_at',
      header: 'Last Activity',
      render: (row: SessionRow) => <RelativeTime timestamp={row.updated_at} />,
    },
  ]

  const violationRows = useMemo(
    () => [...violations].sort((a, b) => Date.parse(b.timestamp) - Date.parse(a.timestamp)) as ViolationRow[],
    [violations],
  )

  const violationColumns = [
    {
      key: 'timestamp',
      header: 'Timestamp',
      render: (row: ViolationRow) => <RelativeTime timestamp={row.timestamp} />,
    },
    {
      key: 'action',
      header: 'Action',
      render: (row: ViolationRow) => (
        <code className="rounded bg-secondary-100 px-1.5 py-0.5 text-xs font-medium text-secondary-800">
          {row.action}
        </code>
      ),
    },
    {
      key: 'policy_scope',
      header: 'Scope',
      render: (row: ViolationRow) => (
        <span className="text-secondary-600">{row.policy_scope}</span>
      ),
    },
    {
      key: 'reason',
      header: 'Reason',
      render: (row: ViolationRow) => (
        <span className="text-secondary-700">{row.reason}</span>
      ),
    },
    {
      key: 'session_id',
      header: 'Session',
      render: (row: ViolationRow) =>
        row.session_id ? (
          <Link
            to={`/sessions/${row.session_id}`}
            className="text-primary-600 hover:text-primary-800 hover:underline"
            onClick={(e) => e.stopPropagation()}
          >
            {row.session_id.slice(0, 8)}...
          </Link>
        ) : (
          <span className="text-secondary-400">&mdash;</span>
        ),
    },
  ]

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
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-semibold text-secondary-900">{machine.name}</h1>
          {machinePolicy && <SessionModeBadge mode={machinePolicy.session_mode} />}
        </div>
        <Button variant="danger" size="sm" onClick={() => setShowRevokeModal(true)}>
          Revoke
        </Button>
      </div>

      {error && (
        <div className="rounded-md bg-danger-50 p-3 text-sm text-danger-700">{error}</div>
      )}

      {/* Connectivity Status Banner */}
      {machine.connectivity_status === 'online' && (
        <div className="rounded-md border-l-4 border-green-500 bg-green-50 px-4 py-3 text-sm text-green-800">
          Machine is online
        </div>
      )}
      {machine.connectivity_status === 'stale' && (
        <div className="rounded-md border-l-4 border-amber-400 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          Machine hasn&apos;t sent a heartbeat recently
        </div>
      )}
      {machine.connectivity_status === 'offline' && (
        <div className="rounded-md border-l-4 border-red-500 bg-red-50 px-4 py-3 text-sm text-red-800">
          Machine is offline
        </div>
      )}

      {/* Tab Navigation */}
      <div className="border-b border-secondary-200 overflow-x-auto">
        <nav className="-mb-px flex gap-6" aria-label="Tabs">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              className={`whitespace-nowrap border-b-2 py-2.5 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'border-primary-500 text-primary-600'
                  : 'border-transparent text-secondary-500 hover:border-secondary-300 hover:text-secondary-700'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Logs Tab */}
      {activeTab === 'logs' && (
        <MachineLogViewer machineId={id!} />
      )}

      {/* Sessions Tab */}
      {activeTab === 'sessions' && (
        <div className="space-y-4">
          {/* State filter */}
          <div className="flex items-end gap-4">
            <div>
              <label className="block text-xs font-medium text-secondary-500 mb-1">State</label>
              <select
                value={sessionStateFilter}
                onChange={(e) => setSessionStateFilter(e.target.value)}
                className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              >
                {sessionStateOptions.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {sessionsError && (
            <div className="rounded-md bg-danger-50 border border-danger-200 p-3 text-sm text-danger-700">
              {sessionsError}
              <button
                type="button"
                className="ml-2 text-sm font-medium text-primary-600 hover:text-primary-700"
                onClick={() => void fetchSessions()}
              >
                Retry
              </button>
            </div>
          )}

          <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
            {sessionsLoading && sessions.length === 0 ? (
              <div className="flex items-center justify-center px-4 py-12">
                <div className="flex items-center gap-2 text-sm text-secondary-500">
                  <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                  </svg>
                  Loading sessions...
                </div>
              </div>
            ) : filteredSessions.length === 0 ? (
              <div className="px-4 py-12 text-center">
                <p className="text-sm text-secondary-500">
                  {sessionStateFilter
                    ? 'No sessions match the current filter.'
                    : 'No sessions found for this machine.'}
                </p>
              </div>
            ) : (
              <Table<SessionRow>
                columns={sessionColumns}
                data={filteredSessions}
                onRowClick={(row) => navigate(`/sessions/${row.id}`)}
              />
            )}
          </div>
        </div>
      )}

      {/* Violations Tab */}
      {activeTab === 'violations' && (
        <div className="space-y-4">
          {violationsError && (
            <div className="rounded-md bg-danger-50 border border-danger-200 p-3 text-sm text-danger-700">
              {violationsError}
              <button
                type="button"
                className="ml-2 text-sm font-medium text-primary-600 hover:text-primary-700"
                onClick={() => void fetchViolations()}
              >
                Retry
              </button>
            </div>
          )}

          <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
            {violationsLoading && violations.length === 0 ? (
              <div className="flex items-center justify-center px-4 py-12">
                <div className="flex items-center gap-2 text-sm text-secondary-500">
                  <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                  </svg>
                  Loading violations...
                </div>
              </div>
            ) : violationRows.length === 0 ? (
              <div className="px-4 py-12 text-center">
                <p className="text-sm text-secondary-500">No policy violations recorded for this machine.</p>
              </div>
            ) : (
              <Table<ViolationRow>
                columns={violationColumns}
                data={violationRows}
              />
            )}
          </div>
        </div>
      )}

      {/* Details Tab */}
      {activeTab === 'details' && (<>

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

      <Card title="Machine Policy">
        {policyLoading && (
          <div className="flex items-center gap-2 text-sm text-secondary-500">
            <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            Loading policy...
          </div>
        )}
        {policyError && (
          <div className="rounded-md bg-danger-50 p-3 text-sm text-danger-700">
            {policyError}
            <button
              type="button"
              className="ml-2 text-sm font-medium text-primary-600 hover:text-primary-700"
              onClick={() => void fetchMachinePolicy()}
            >
              Retry
            </button>
          </div>
        )}
        {machinePolicy && !policyLoading && (
          <PolicyEditor
            policy={machinePolicy}
            onSave={async (data) => {
              if (!id) return
              const updated = await updateMachinePolicy(id, data)
              setMachinePolicy(updated)
            }}
            showSessionMode
          />
        )}
      </Card>

      {machine.repos.length > 0 && (
        <Card title="Repository Policies" subtitle="Override machine-level policy per repository">
          <div className="divide-y divide-secondary-100">
            {machine.repos.map((repo) => (
              <div key={repo.id} className="py-3">
                <div className="flex items-center justify-between">
                  <div>
                    <span className="text-sm font-medium text-secondary-900">{repo.repo_name}</span>
                    <span className="ml-2 text-xs text-secondary-400">{repo.repo_path}</span>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      if (expandedRepoPolicy === repo.repo_path) {
                        setExpandedRepoPolicy(null)
                      } else {
                        setExpandedRepoPolicy(repo.repo_path)
                        if (!repoPolicies[repo.repo_path]) {
                          void fetchRepoPolicy(repo.repo_path)
                        }
                      }
                    }}
                  >
                    {expandedRepoPolicy === repo.repo_path ? 'Hide Policy' : 'Edit Policy'}
                  </Button>
                </div>
                {expandedRepoPolicy === repo.repo_path && (
                  <div className="mt-4 rounded-md border border-secondary-100 bg-secondary-50/50 p-4">
                    {repoPolicyLoading === repo.repo_path ? (
                      <div className="flex items-center gap-2 text-sm text-secondary-500">
                        <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                        </svg>
                        Loading repo policy...
                      </div>
                    ) : repoPolicies[repo.repo_path] ? (
                      <PolicyEditor
                        policy={repoPolicies[repo.repo_path]}
                        onSave={async (data) => {
                          if (!id) return
                          const updated = await updateRepoPolicy(id, repo.repo_path, data)
                          setRepoPolicies((prev) => ({ ...prev, [repo.repo_path]: updated }))
                        }}
                        showSessionMode={false}
                      />
                    ) : (
                      <p className="text-sm text-secondary-500">
                        No repository-level policy configured. The machine policy applies.
                      </p>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        </Card>
      )}

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

      </>)}

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
