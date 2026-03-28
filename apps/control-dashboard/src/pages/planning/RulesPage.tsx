import { useState, useEffect, useCallback, useMemo } from 'react'
import type { Rule } from '../../api/planning'
import { getRules } from '../../api/planning'
import { Table } from '../../components/ui/Table'
import { Badge } from '../../components/ui/Badge'
import { FormInput } from '../../components/ui/FormInput'
import { Select } from '../../components/ui/Select'

const scopeOptions = [
  { value: '', label: 'All Scopes' },
  { value: 'repo', label: 'Repo' },
  { value: 'package', label: 'Package' },
  { value: 'subsystem', label: 'Subsystem' },
]

const protectionConfig: Record<string, { variant: 'error' | 'info' | 'offline'; label: string }> = {
  protected: { variant: 'error', label: 'Protected' },
  standard: { variant: 'info', label: 'Standard' },
  advisory: { variant: 'offline', label: 'Advisory' },
}

const columns = [
  {
    key: 'name',
    header: 'Name',
    render: (row: Rule) => (
      <span className="font-medium text-secondary-900">{row.name}</span>
    ),
  },
  {
    key: 'scope',
    header: 'Scope',
    render: (row: Rule) => (
      <span className="text-sm capitalize text-secondary-600">{row.scope}</span>
    ),
  },
  {
    key: 'description',
    header: 'Description',
    render: (row: Rule) => (
      <span className="text-sm text-secondary-600">{row.description}</span>
    ),
  },
  {
    key: 'protection_level',
    header: 'Protection Level',
    render: (row: Rule) => {
      const config = protectionConfig[row.protection_level] ?? {
        variant: 'offline' as const,
        label: row.protection_level,
      }
      return <Badge variant={config.variant}>{config.label}</Badge>
    },
  },
]

export default function RulesPage() {
  const [rules, setRules] = useState<Rule[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [scopeFilter, setScopeFilter] = useState('')

  const fetchRules = useCallback(async (scope: string) => {
    try {
      setLoading(true)
      const data = await getRules(scope || undefined)
      setRules(data)
      setError(null)
    } catch {
      setError('Failed to load rules')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void fetchRules(scopeFilter)
  }, [fetchRules, scopeFilter])

  const filteredRules = useMemo(() => {
    if (!searchQuery.trim()) return rules
    const query = searchQuery.toLowerCase()
    return rules.filter(
      (r) =>
        r.name.toLowerCase().includes(query) ||
        r.description.toLowerCase().includes(query),
    )
  }, [rules, searchQuery])

  const tableData = useMemo(
    () => filteredRules as (Rule & Record<string, unknown>)[],
    [filteredRules],
  )

  if (error && !loading && rules.length === 0) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold text-secondary-900">Rules</h2>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-8 py-12 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => void fetchRules(scopeFilter)}
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
        <h2 className="text-2xl font-semibold text-secondary-900">Rules</h2>
      </div>

      {/* Filters */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <FormInput
          label="Search"
          placeholder="Search rules by name or description..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
        <Select
          label="Scope"
          options={scopeOptions}
          value={scopeFilter}
          onChange={setScopeFilter}
          placeholder="All Scopes"
        />
      </div>

      {/* Table */}
      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="text-sm text-secondary-500">Loading rules...</div>
          </div>
        ) : filteredRules.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">
              {searchQuery || scopeFilter
                ? 'No rules match the current filters.'
                : 'No active rules found.'}
            </p>
          </div>
        ) : (
          <Table columns={columns} data={tableData} />
        )}
      </div>
    </div>
  )
}
