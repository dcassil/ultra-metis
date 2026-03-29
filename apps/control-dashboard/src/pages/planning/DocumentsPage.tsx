import { useState, useEffect, useCallback, useRef, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import type { PlanningDocument } from '../../api/planning'
import { listDocuments, searchDocuments } from '../../api/planning'
import { Table } from '../../components/ui/Table'
import { Select } from '../../components/ui/Select'
import { FormInput } from '../../components/ui/FormInput'
import { DocumentTypeBadge } from '../../components/planning/DocumentTypeBadge'
import { PhaseBadge } from '../../components/planning/PhaseBadge'
import { ShortCodeLink } from '../../components/planning/ShortCodeLink'

const typeOptions = [
  { value: '', label: 'All Types' },
  { value: 'vision', label: 'Vision' },
  { value: 'strategy', label: 'Strategy' },
  { value: 'initiative', label: 'Initiative' },
  { value: 'task', label: 'Task' },
  { value: 'adr', label: 'ADR' },
]

const phaseOptions = [
  { value: '', label: 'All Phases' },
  { value: 'draft', label: 'Draft' },
  { value: 'review', label: 'Review' },
  { value: 'published', label: 'Published' },
  { value: 'shaping', label: 'Shaping' },
  { value: 'discovery', label: 'Discovery' },
  { value: 'design', label: 'Design' },
  { value: 'ready', label: 'Ready' },
  { value: 'decompose', label: 'Decompose' },
  { value: 'active', label: 'Active' },
  { value: 'completed', label: 'Completed' },
  { value: 'todo', label: 'To Do' },
  { value: 'discussion', label: 'Discussion' },
  { value: 'decided', label: 'Decided' },
  { value: 'superseded', label: 'Superseded' },
]

const columns = [
  {
    key: 'short_code',
    header: 'Short Code',
    render: (row: PlanningDocument) => <ShortCodeLink shortCode={row.short_code} />,
  },
  {
    key: 'title',
    header: 'Title',
    render: (row: PlanningDocument) => (
      <span className="font-medium text-secondary-900">{row.title}</span>
    ),
  },
  {
    key: 'document_type',
    header: 'Type',
    render: (row: PlanningDocument) => <DocumentTypeBadge type={row.document_type} />,
  },
  {
    key: 'phase',
    header: 'Phase',
    render: (row: PlanningDocument) => <PhaseBadge phase={row.phase} />,
  },
  {
    key: 'parent_id',
    header: 'Parent',
    render: (row: PlanningDocument) =>
      row.parent_id ? (
        <ShortCodeLink shortCode={row.parent_id} />
      ) : (
        <span className="text-secondary-400">&mdash;</span>
      ),
  },
]

export default function DocumentsPage() {
  const navigate = useNavigate()
  const [documents, setDocuments] = useState<PlanningDocument[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [typeFilter, setTypeFilter] = useState('')
  const [phaseFilter, setPhaseFilter] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const fetchDocuments = useCallback(async (query: string, docType: string, phase: string) => {
    try {
      setLoading(true)
      let data: PlanningDocument[]
      if (query.trim()) {
        data = await searchDocuments(query.trim(), docType || undefined)
        // Client-side phase filter for search results since search API doesn't support phase param
        if (phase) {
          data = data.filter((d) => d.phase === phase)
        }
      } else {
        data = await listDocuments({
          document_type: docType || undefined,
          phase: phase || undefined,
        })
      }
      setDocuments(data)
      setError(null)
    } catch {
      setError('Failed to load documents')
    } finally {
      setLoading(false)
    }
  }, [])

  // Debounced search
  const debouncedFetch = useCallback(
    (query: string, docType: string, phase: string) => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current)
      }
      debounceRef.current = setTimeout(() => {
        void fetchDocuments(query, docType, phase)
      }, 300)
    },
    [fetchDocuments],
  )

  // Initial load and filter changes
  useEffect(() => {
    void fetchDocuments(searchQuery, typeFilter, phaseFilter)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [typeFilter, phaseFilter])

  const handleSearchChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value
      setSearchQuery(value)
      debouncedFetch(value, typeFilter, phaseFilter)
    },
    [debouncedFetch, typeFilter, phaseFilter],
  )

  const handleRowClick = useCallback(
    (row: PlanningDocument) => {
      navigate(`/planning/documents/${row.short_code}`)
    },
    [navigate],
  )

  // Memoize data cast for Table generic
  const tableData = useMemo(() => documents as (PlanningDocument & Record<string, unknown>)[], [documents])

  if (error && !loading && documents.length === 0) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold text-secondary-900">Documents</h2>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-4 py-12 sm:px-8 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => void fetchDocuments(searchQuery, typeFilter, phaseFilter)}
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
        <h2 className="text-2xl font-semibold text-secondary-900">Documents</h2>
      </div>

      {/* Filters */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <FormInput
          label="Search"
          placeholder="Search documents..."
          value={searchQuery}
          onChange={handleSearchChange}
        />
        <Select
          label="Type"
          options={typeOptions}
          value={typeFilter}
          onChange={setTypeFilter}
          placeholder="All Types"
        />
        <Select
          label="Phase"
          options={phaseOptions}
          value={phaseFilter}
          onChange={setPhaseFilter}
          placeholder="All Phases"
        />
      </div>

      {/* Table */}
      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="text-sm text-secondary-500">Loading documents...</div>
          </div>
        ) : documents.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">
              {searchQuery || typeFilter || phaseFilter
                ? 'No documents match the current filters.'
                : 'No documents found.'}
            </p>
          </div>
        ) : (
          <Table
            columns={columns}
            data={tableData}
            onRowClick={handleRowClick}
            mobileCardConfig={{
              headerColumn: 'title',
              badgeColumn: 'document_type',
              bodyColumns: ['phase', 'parent_id'],
            }}
          />
        )}
      </div>
    </div>
  )
}
