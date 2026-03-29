import { useState, useEffect, useCallback, useMemo } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { listSessions } from '../api/sessions'
import type { SessionResponse } from '../api/sessions'
import { Button } from '../components/ui/Button'
import { Table } from '../components/ui/Table'
import { SessionStateBadge } from '../components/SessionStateBadge'
import { RelativeTime } from '../components/RelativeTime'

type SortKey = 'created_at' | 'updated_at' | 'state'

const stateOptions = [
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

export default function SessionsPage() {
  const navigate = useNavigate()
  const [sessions, setSessions] = useState<SessionResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Filter state
  const [stateFilter, setStateFilter] = useState('')
  const [machineFilter, setMachineFilter] = useState('')
  const [repoSearch, setRepoSearch] = useState('')
  const [sortBy, setSortBy] = useState<SortKey>('created_at')

  const fetchSessions = useCallback(async () => {
    try {
      const params: Record<string, string> = {}
      if (stateFilter) params.state = stateFilter
      if (machineFilter) params.machine_id = machineFilter
      const data = await listSessions(params)
      setSessions(data.sessions)
      setError(null)
    } catch {
      setError('Failed to load sessions')
    } finally {
      setLoading(false)
    }
  }, [stateFilter, machineFilter])

  useEffect(() => {
    void fetchSessions()
    const interval = setInterval(() => void fetchSessions(), 5_000)
    return () => clearInterval(interval)
  }, [fetchSessions])

  // Derive unique machine IDs for the machine filter dropdown
  const machineIds = useMemo(
    () => Array.from(new Set(sessions.map((s) => s.machine_id))).sort(),
    [sessions],
  )

  // Filter and sort
  const filteredSessions = useMemo(() => {
    let result = sessions

    if (repoSearch.trim()) {
      const q = repoSearch.trim().toLowerCase()
      result = result.filter((s) => s.repo_path.toLowerCase().includes(q))
    }

    result = [...result].sort((a, b) => {
      if (sortBy === 'created_at') {
        return Date.parse(b.created_at) - Date.parse(a.created_at)
      }
      if (sortBy === 'updated_at') {
        return Date.parse(b.updated_at) - Date.parse(a.updated_at)
      }
      // sort by state
      return a.state.localeCompare(b.state)
    })

    return result as SessionRow[]
  }, [sessions, repoSearch, sortBy])

  const columns = [
    {
      key: 'title',
      header: 'Title',
      render: (row: SessionRow) => (
        <span className="inline-flex items-center gap-2 font-medium text-secondary-900">
          {row.title}
          {row.state === 'waiting_for_input' && (
            <span className="inline-flex items-center rounded-full bg-warning-100 px-2 py-0.5 text-xs font-semibold text-warning-800">
              Needs Approval
            </span>
          )}
        </span>
      ),
    },
    {
      key: 'machine_id',
      header: 'Machine',
      render: (row: SessionRow) => (
        <Link
          to={`/machines/${row.machine_id}`}
          className="text-primary-600 hover:text-primary-800 hover:underline"
          onClick={(e) => e.stopPropagation()}
        >
          {row.machine_id}
        </Link>
      ),
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
      key: 'state',
      header: 'State',
      render: (row: SessionRow) => (
        <span className="inline-flex items-center gap-2">
          {row.state === 'waiting_for_input' && (
            <span className="relative flex h-2.5 w-2.5">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-red-400 opacity-75" />
              <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-red-500" />
            </span>
          )}
          {row.state === 'running' && (
            <span className="relative flex h-2.5 w-2.5">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
              <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-green-500" />
            </span>
          )}
          <SessionStateBadge state={row.state} />
        </span>
      ),
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
    {
      key: 'autonomy_level',
      header: 'Autonomy',
      render: (row: SessionRow) => (
        <span className="capitalize text-secondary-600">{row.autonomy_level}</span>
      ),
    },
  ]

  if (loading) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="text-sm text-secondary-500">Loading sessions...</div>
      </div>
    )
  }

  if (error && sessions.length === 0) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-semibold text-secondary-900">Sessions</h2>
          <Link to="/sessions/new">
            <Button>Start Session</Button>
          </Link>
        </div>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-4 py-12 sm:px-8 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => {
                setLoading(true)
                void fetchSessions()
              }}
              className="mt-3 text-sm font-medium text-primary-600 hover:text-primary-700"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-semibold text-secondary-900">Sessions</h2>
        <Link to="/sessions/new">
          <Button>Start Session</Button>
        </Link>
      </div>

      {error && (
        <div className="rounded-md bg-danger-50 border border-danger-200 p-3 text-sm text-danger-700">
          {error}
        </div>
      )}

      {/* Filters */}
      <div className="flex flex-wrap items-end gap-4">
        <div>
          <label className="block text-xs font-medium text-secondary-500 mb-1">State</label>
          <select
            value={stateFilter}
            onChange={(e) => setStateFilter(e.target.value)}
            className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          >
            {stateOptions.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-xs font-medium text-secondary-500 mb-1">Machine</label>
          <select
            value={machineFilter}
            onChange={(e) => setMachineFilter(e.target.value)}
            className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          >
            <option value="">All Machines</option>
            {machineIds.map((mid) => (
              <option key={mid} value={mid}>
                {mid}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-xs font-medium text-secondary-500 mb-1">Repo</label>
          <input
            type="text"
            value={repoSearch}
            onChange={(e) => setRepoSearch(e.target.value)}
            placeholder="Search repo..."
            className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>

        <div>
          <label className="block text-xs font-medium text-secondary-500 mb-1">Sort by</label>
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as SortKey)}
            className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          >
            <option value="created_at">Created (newest)</option>
            <option value="updated_at">Last Activity</option>
            <option value="state">State</option>
          </select>
        </div>
      </div>

      {/* Table */}
      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {filteredSessions.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">
              {sessions.length === 0
                ? 'No sessions yet. Start a new session to get going.'
                : 'No sessions match the current filters.'}
            </p>
          </div>
        ) : (
          <Table<SessionRow>
            columns={columns}
            data={filteredSessions}
            onRowClick={(row) => navigate(`/sessions/${row.id}`)}
            mobileCardConfig={{
              headerColumn: 'title',
              badgeColumn: 'state',
              bodyColumns: ['machine_id', 'elapsed', 'updated_at'],
            }}
          />
        )}
      </div>
    </div>
  )
}
