import { useState, useEffect, useCallback, useRef, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import type { SessionResponse } from '../api/sessions'
import { listSessions } from '../api/sessions'
import { Table } from '../components/ui/Table'
import { Select } from '../components/ui/Select'
import { FormInput } from '../components/ui/FormInput'
import { Button } from '../components/ui/Button'
import { Badge } from '../components/ui/Badge'
import { SessionStateBadge } from '../components/SessionStateBadge'
import { RelativeTime } from '../components/RelativeTime'

const PAGE_SIZE = 25

const TERMINAL_STATES = ['completed', 'failed', 'stopped']

const outcomeOptions = [
  { value: '', label: 'All Outcomes' },
  { value: 'success', label: 'Success' },
  { value: 'partial', label: 'Partial' },
  { value: 'failure', label: 'Failure' },
]

const sortOptions = [
  { value: 'newest', label: 'Newest First' },
  { value: 'oldest', label: 'Oldest First' },
]

type OutcomeBadgeVariant = 'online' | 'error' | 'pending' | 'offline'

function outcomeVariant(outcome: string | null | undefined): OutcomeBadgeVariant {
  switch (outcome) {
    case 'success':
      return 'online'
    case 'failure':
      return 'error'
    case 'partial':
      return 'pending'
    default:
      return 'offline'
  }
}

function outcomeLabel(outcome: string | null | undefined): string {
  switch (outcome) {
    case 'success':
      return 'Success'
    case 'failure':
      return 'Failure'
    case 'partial':
      return 'Partial'
    default:
      return 'None'
  }
}

function formatDuration(startedAt?: string, completedAt?: string): string {
  if (!startedAt) return '\u2014'
  const start = new Date(startedAt).getTime()
  const end = completedAt ? new Date(completedAt).getTime() : Date.now()
  const diffMs = end - start
  if (diffMs < 0) return '\u2014'

  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ${seconds % 60}s`
  const hours = Math.floor(minutes / 60)
  return `${hours}h ${minutes % 60}m`
}

function truncateId(id: string, maxLen = 12): string {
  if (id.length <= maxLen) return id
  return id.slice(0, maxLen) + '\u2026'
}

function truncatePath(path: string, maxLen = 30): string {
  if (path.length <= maxLen) return path
  return '\u2026' + path.slice(path.length - maxLen + 1)
}

export default function HistoryPage() {
  const navigate = useNavigate()
  const [sessions, setSessions] = useState<SessionResponse[]>([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(true)
  const [loadingMore, setLoadingMore] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [outcomeFilter, setOutcomeFilter] = useState('')
  const [sortOrder, setSortOrder] = useState('newest')
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const fetchSessions = useCallback(
    async (query: string, outcome: string, sort: string, offset = 0, append = false) => {
      try {
        if (append) {
          setLoadingMore(true)
        } else {
          setLoading(true)
        }
        const data = await listSessions({
          search: query.trim() || undefined,
          outcome: outcome || undefined,
          sort_by: 'created_at',
          sort_order: sort === 'newest' ? 'desc' : 'asc',
          limit: PAGE_SIZE,
          offset,
        })
        // Client-side filter for terminal states since API may not support comma-separated state values
        const filtered = data.sessions.filter((s) => TERMINAL_STATES.includes(s.state))
        if (append) {
          setSessions((prev) => [...prev, ...filtered])
        } else {
          setSessions(filtered)
        }
        setTotal(data.total)
        setError(null)
      } catch {
        setError('Failed to load session history')
      } finally {
        setLoading(false)
        setLoadingMore(false)
      }
    },
    [],
  )

  // Debounced search
  const debouncedFetch = useCallback(
    (query: string, outcome: string, sort: string) => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current)
      }
      debounceRef.current = setTimeout(() => {
        void fetchSessions(query, outcome, sort)
      }, 300)
    },
    [fetchSessions],
  )

  // Initial load and filter changes (non-search)
  useEffect(() => {
    void fetchSessions(searchQuery, outcomeFilter, sortOrder)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [outcomeFilter, sortOrder])

  const handleSearchChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value
      setSearchQuery(value)
      debouncedFetch(value, outcomeFilter, sortOrder)
    },
    [debouncedFetch, outcomeFilter, sortOrder],
  )

  const handleLoadMore = useCallback(() => {
    void fetchSessions(searchQuery, outcomeFilter, sortOrder, sessions.length, true)
  }, [fetchSessions, searchQuery, outcomeFilter, sortOrder, sessions.length])

  const handleRowClick = useCallback(
    (row: SessionResponse) => {
      navigate(`/history/${row.id}`)
    },
    [navigate],
  )

  const columns = useMemo(
    () => [
      {
        key: 'title',
        header: 'Title',
        render: (row: SessionResponse) => (
          <span className="font-medium text-secondary-900">{row.title || 'Untitled'}</span>
        ),
      },
      {
        key: 'machine_id',
        header: 'Machine',
        render: (row: SessionResponse) => (
          <span className="font-mono text-xs text-secondary-600" title={row.machine_id}>
            {truncateId(row.machine_id)}
          </span>
        ),
      },
      {
        key: 'repo_path',
        header: 'Repo',
        render: (row: SessionResponse) => (
          <span className="text-xs text-secondary-600" title={row.repo_path}>
            {truncatePath(row.repo_path)}
          </span>
        ),
      },
      {
        key: 'state',
        header: 'State',
        render: (row: SessionResponse) => <SessionStateBadge state={row.state} />,
      },
      {
        key: 'outcome_status',
        header: 'Outcome',
        render: (row: SessionResponse) => (
          <Badge variant={outcomeVariant(row.outcome_status)}>
            {outcomeLabel(row.outcome_status)}
          </Badge>
        ),
      },
      {
        key: 'duration',
        header: 'Duration',
        render: (row: SessionResponse) => (
          <span className="text-xs text-secondary-600">
            {formatDuration(row.started_at, row.completed_at)}
          </span>
        ),
      },
      {
        key: 'created_at',
        header: 'Date',
        render: (row: SessionResponse) => (
          <span className="text-xs text-secondary-600">
            <RelativeTime timestamp={row.created_at} />
          </span>
        ),
      },
    ],
    [],
  )

  // Memoize data cast for Table generic
  const tableData = useMemo(
    () => sessions as (SessionResponse & Record<string, unknown>)[],
    [sessions],
  )

  const hasMore = sessions.length < total

  if (error && !loading && sessions.length === 0) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold text-secondary-900">Session History</h2>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-4 py-12 sm:px-8 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => void fetchSessions(searchQuery, outcomeFilter, sortOrder)}
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
        <h2 className="text-2xl font-semibold text-secondary-900">Session History</h2>
      </div>

      {/* Filters */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <FormInput
          label="Search"
          placeholder="Search sessions..."
          value={searchQuery}
          onChange={handleSearchChange}
        />
        <Select
          label="Outcome"
          options={outcomeOptions}
          value={outcomeFilter}
          onChange={setOutcomeFilter}
          placeholder="All Outcomes"
        />
        <Select
          label="Sort"
          options={sortOptions}
          value={sortOrder}
          onChange={setSortOrder}
          placeholder="Newest First"
        />
      </div>

      {/* Table */}
      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="text-sm text-secondary-500">Loading session history...</div>
          </div>
        ) : sessions.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">
              {searchQuery || outcomeFilter
                ? 'No sessions match the current filters.'
                : 'No session history found.'}
            </p>
          </div>
        ) : (
          <Table
            columns={columns}
            data={tableData}
            onRowClick={handleRowClick}
            mobileCardConfig={{
              headerColumn: 'title',
              badgeColumn: 'outcome_status',
              bodyColumns: ['machine_id', 'duration', 'created_at'],
            }}
          />
        )}
      </div>

      {/* Load More */}
      {!loading && hasMore && (
        <div className="flex justify-center">
          <Button variant="secondary" size="sm" loading={loadingMore} onClick={handleLoadMore}>
            Load More
          </Button>
        </div>
      )}
    </div>
  )
}
