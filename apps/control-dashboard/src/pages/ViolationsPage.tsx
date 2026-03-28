import { useState, useEffect, useCallback, useMemo } from 'react'
import { Link } from 'react-router-dom'
import { listViolations } from '../api/policies'
import type { PolicyViolationRecord } from '../api/policies'
import { listMachines } from '../api/machines'
import type { Machine } from '../api/machines'
import { RelativeTime } from '../components/RelativeTime'

const PAGE_SIZE = 25

export default function ViolationsPage() {
  const [violations, setViolations] = useState<PolicyViolationRecord[]>([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Machines for filter dropdown
  const [machines, setMachines] = useState<Machine[]>([])

  // Filters
  const [machineFilter, setMachineFilter] = useState('')
  const [sessionFilter, setSessionFilter] = useState('')

  // Pagination
  const [page, setPage] = useState(0)

  const fetchMachines = useCallback(async () => {
    try {
      const data = await listMachines()
      setMachines(data)
    } catch {
      // Non-critical — filter dropdown just won't populate
    }
  }, [])

  const fetchViolations = useCallback(async () => {
    try {
      const params: Record<string, string | number> = {
        limit: PAGE_SIZE,
        offset: page * PAGE_SIZE,
      }
      if (machineFilter) params.machine_id = machineFilter
      if (sessionFilter.trim()) params.session_id = sessionFilter.trim()

      const data = await listViolations(params)
      setViolations(data.violations)
      setTotal(data.total)
      setError(null)
    } catch {
      setError('Failed to load violations')
    } finally {
      setLoading(false)
    }
  }, [machineFilter, sessionFilter, page])

  useEffect(() => {
    void fetchMachines()
  }, [fetchMachines])

  useEffect(() => {
    setLoading(true)
    void fetchViolations()
    const interval = setInterval(() => void fetchViolations(), 30_000)
    return () => clearInterval(interval)
  }, [fetchViolations])

  // Reset to first page when filters change
  const handleMachineFilterChange = (value: string) => {
    setMachineFilter(value)
    setPage(0)
  }

  const handleSessionFilterChange = (value: string) => {
    setSessionFilter(value)
    setPage(0)
  }

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))

  const machineNameMap = useMemo(() => {
    const map = new Map<string, string>()
    for (const m of machines) {
      map.set(m.id, m.name)
    }
    return map
  }, [machines])

  if (loading && violations.length === 0) {
    return (
      <div className="flex items-center justify-center py-24">
        <div className="text-sm text-secondary-500">Loading violations...</div>
      </div>
    )
  }

  if (error && violations.length === 0) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold text-secondary-900">Policy Violations</h2>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-8 py-12 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => {
                setLoading(true)
                void fetchViolations()
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
      <h2 className="text-2xl font-semibold text-secondary-900">Policy Violations</h2>

      {error && (
        <div className="rounded-md bg-danger-50 border border-danger-200 p-3 text-sm text-danger-700">
          {error}
        </div>
      )}

      {/* Filters */}
      <div className="flex flex-wrap items-end gap-4">
        <div>
          <label className="block text-xs font-medium text-secondary-500 mb-1">Machine</label>
          <select
            value={machineFilter}
            onChange={(e) => handleMachineFilterChange(e.target.value)}
            className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          >
            <option value="">All Machines</option>
            {machines.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-xs font-medium text-secondary-500 mb-1">Session ID</label>
          <input
            type="text"
            value={sessionFilter}
            onChange={(e) => handleSessionFilterChange(e.target.value)}
            placeholder="Filter by session..."
            className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>
      </div>

      {/* Table */}
      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {violations.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">No policy violations recorded.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-secondary-200">
              <thead className="bg-secondary-50">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-secondary-500">
                    Timestamp
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-secondary-500">
                    Action
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-secondary-500">
                    Scope
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-secondary-500">
                    Reason
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-secondary-500">
                    Machine
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-secondary-500">
                    Session
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-secondary-200 bg-white">
                {violations.map((v) => (
                  <tr key={v.id} className="text-sm text-secondary-900">
                    <td className="whitespace-nowrap px-4 py-3">
                      <RelativeTime timestamp={v.timestamp} />
                    </td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <code className="rounded bg-secondary-100 px-1.5 py-0.5 text-xs font-mono text-secondary-700">
                        {v.action}
                      </code>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 capitalize">
                      {v.policy_scope}
                    </td>
                    <td className="px-4 py-3 max-w-xs truncate" title={v.reason}>
                      {v.reason}
                    </td>
                    <td className="whitespace-nowrap px-4 py-3">
                      <Link
                        to={`/machines/${v.machine_id}`}
                        className="text-primary-600 hover:text-primary-800"
                      >
                        {machineNameMap.get(v.machine_id) ?? v.machine_id}
                      </Link>
                    </td>
                    <td className="whitespace-nowrap px-4 py-3">
                      {v.session_id ? (
                        <Link
                          to={`/sessions/${v.session_id}`}
                          className="text-primary-600 hover:text-primary-800"
                        >
                          {v.session_id.slice(0, 8)}...
                        </Link>
                      ) : (
                        <span className="text-secondary-400">--</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Pagination */}
      {total > PAGE_SIZE && (
        <div className="flex items-center justify-between">
          <p className="text-sm text-secondary-500">
            Showing {page * PAGE_SIZE + 1}–{Math.min((page + 1) * PAGE_SIZE, total)} of {total} violations
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              disabled={page === 0}
              onClick={() => setPage((p) => p - 1)}
              className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm hover:bg-secondary-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Previous
            </button>
            <button
              type="button"
              disabled={page >= totalPages - 1}
              onClick={() => setPage((p) => p + 1)}
              className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm shadow-sm hover:bg-secondary-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
