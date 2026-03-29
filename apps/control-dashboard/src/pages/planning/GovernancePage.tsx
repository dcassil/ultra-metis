import { useState, useEffect, useCallback, useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'
import type { PlanningDocument, QualityRecord, Rule } from '../../api/planning'
import { listDocuments, getQualityRecords, getRules } from '../../api/planning'
import { Table } from '../../components/ui/Table'
import { Card } from '../../components/ui/Card'
import { Badge } from '../../components/ui/Badge'
import { DocumentTypeBadge } from '../../components/planning/DocumentTypeBadge'
import { ShortCodeLink } from '../../components/planning/ShortCodeLink'
import { FormInput } from '../../components/ui/FormInput'
import { Select } from '../../components/ui/Select'

// --- Quality tab types and helpers ---

interface DocumentWithQuality extends PlanningDocument {
  quality_records: QualityRecord[]
  overall_status: 'pass' | 'fail' | 'pending' | 'none'
}

function deriveOverallStatus(records: QualityRecord[]): 'pass' | 'fail' | 'pending' | 'none' {
  if (records.length === 0) return 'none'
  if (records.some((r) => r.gate_status === 'fail')) return 'fail'
  if (records.some((r) => r.gate_status === 'pending')) return 'pending'
  return 'pass'
}

const qualityStatusConfig: Record<string, { variant: 'online' | 'offline' | 'pending' | 'error'; label: string }> = {
  pass: { variant: 'online', label: 'Pass' },
  fail: { variant: 'error', label: 'Fail' },
  pending: { variant: 'pending', label: 'Pending' },
  none: { variant: 'offline', label: 'No Records' },
}

const qualityColumns = [
  {
    key: 'short_code',
    header: 'Short Code',
    render: (row: DocumentWithQuality) => <ShortCodeLink shortCode={row.short_code} />,
  },
  {
    key: 'title',
    header: 'Title',
    render: (row: DocumentWithQuality) => (
      <span className="font-medium text-secondary-900">{row.title}</span>
    ),
  },
  {
    key: 'document_type',
    header: 'Type',
    render: (row: DocumentWithQuality) => <DocumentTypeBadge type={row.document_type} />,
  },
  {
    key: 'quality_status',
    header: 'Quality Status',
    render: (row: DocumentWithQuality) => {
      const config = qualityStatusConfig[row.overall_status]
      return <Badge variant={config.variant}>{config.label}</Badge>
    },
  },
  {
    key: 'records_count',
    header: 'Records',
    render: (row: DocumentWithQuality) => (
      <span className="text-sm text-secondary-600">{row.quality_records.length}</span>
    ),
  },
]

// --- Rules tab types and helpers ---

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

const rulesColumns = [
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

// --- Tab definitions ---

type TabId = 'quality' | 'rules'

const TABS: { id: TabId; label: string }[] = [
  { id: 'quality', label: 'Quality Gates' },
  { id: 'rules', label: 'Rules' },
]

const VALID_TABS: TabId[] = ['quality', 'rules']

// --- Component ---

export default function GovernancePage() {
  const [searchParams, setSearchParams] = useSearchParams()

  const initialTab = searchParams.get('tab')
  const activeTab: TabId =
    initialTab && VALID_TABS.includes(initialTab as TabId)
      ? (initialTab as TabId)
      : 'quality'

  const handleTabChange = (tab: TabId) => {
    setSearchParams({ tab }, { replace: true })
  }

  // --- Quality state ---
  const [documents, setDocuments] = useState<DocumentWithQuality[]>([])
  const [qualityLoading, setQualityLoading] = useState(true)
  const [qualityError, setQualityError] = useState<string | null>(null)

  const fetchQualityData = useCallback(async () => {
    try {
      setQualityLoading(true)
      const docs = await listDocuments()

      const docsWithQuality: DocumentWithQuality[] = await Promise.all(
        docs.map(async (doc) => {
          let records: QualityRecord[] = []
          try {
            records = await getQualityRecords(doc.short_code)
          } catch {
            // Quality records may not exist for this document
          }
          return {
            ...doc,
            quality_records: records,
            overall_status: deriveOverallStatus(records),
          }
        }),
      )

      setDocuments(docsWithQuality)
      setQualityError(null)
    } catch {
      setQualityError('Failed to load quality data')
    } finally {
      setQualityLoading(false)
    }
  }, [])

  useEffect(() => {
    void fetchQualityData()
  }, [fetchQualityData])

  const qualitySummary = useMemo(() => {
    const total = documents.length
    const withRecords = documents.filter((d) => d.quality_records.length > 0).length
    const passCount = documents.filter((d) => d.overall_status === 'pass').length
    const failCount = documents.filter((d) => d.overall_status === 'fail').length
    return { total, withRecords, passCount, failCount }
  }, [documents])

  const qualityTableData = useMemo(
    () => documents as (DocumentWithQuality & Record<string, unknown>)[],
    [documents],
  )

  // --- Rules state ---
  const [rules, setRules] = useState<Rule[]>([])
  const [rulesLoading, setRulesLoading] = useState(true)
  const [rulesError, setRulesError] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [scopeFilter, setScopeFilter] = useState('')

  const fetchRules = useCallback(async (scope: string) => {
    try {
      setRulesLoading(true)
      const data = await getRules(scope || undefined)
      setRules(data)
      setRulesError(null)
    } catch {
      setRulesError('Failed to load rules')
    } finally {
      setRulesLoading(false)
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

  const rulesTableData = useMemo(
    () => filteredRules as (Rule & Record<string, unknown>)[],
    [filteredRules],
  )

  // --- Render ---

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-semibold text-secondary-900">Governance</h2>
      </div>

      {/* Tab Navigation */}
      <div className="border-b border-secondary-200 overflow-x-auto">
        <nav className="-mb-px flex gap-6" aria-label="Tabs">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => handleTabChange(tab.id)}
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

      {/* Quality Gates Tab */}
      {activeTab === 'quality' && (
        <>
          {qualityError && !qualityLoading && documents.length === 0 ? (
            <div className="flex items-center justify-center py-24">
              <div className="rounded-lg border border-danger-200 bg-danger-50 px-8 py-12 text-center">
                <p className="text-sm text-danger-700">{qualityError}</p>
                <button
                  type="button"
                  onClick={() => void fetchQualityData()}
                  className="mt-3 text-sm font-medium text-primary-600 hover:text-primary-700"
                >
                  Retry
                </button>
              </div>
            </div>
          ) : (
            <>
              {/* Summary Cards */}
              {!qualityLoading && (
                <div className="grid grid-cols-1 gap-4 sm:grid-cols-4">
                  <Card>
                    <div className="text-center">
                      <p className="text-2xl font-bold text-secondary-900">{qualitySummary.total}</p>
                      <p className="text-sm text-secondary-500">Total Documents</p>
                    </div>
                  </Card>
                  <Card>
                    <div className="text-center">
                      <p className="text-2xl font-bold text-secondary-900">{qualitySummary.withRecords}</p>
                      <p className="text-sm text-secondary-500">Quality Records</p>
                    </div>
                  </Card>
                  <Card>
                    <div className="text-center">
                      <p className="text-2xl font-bold text-success-600">{qualitySummary.passCount}</p>
                      <p className="text-sm text-secondary-500">Passing</p>
                    </div>
                  </Card>
                  <Card>
                    <div className="text-center">
                      <p className="text-2xl font-bold text-danger-600">{qualitySummary.failCount}</p>
                      <p className="text-sm text-secondary-500">Failing</p>
                    </div>
                  </Card>
                </div>
              )}

              {/* Table */}
              <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
                {qualityLoading ? (
                  <div className="flex items-center justify-center py-12">
                    <div className="text-sm text-secondary-500">Loading quality data...</div>
                  </div>
                ) : documents.length === 0 ? (
                  <div className="px-4 py-12 text-center">
                    <p className="text-sm text-secondary-500">
                      No quality records found. Quality gates will appear here when configured.
                    </p>
                  </div>
                ) : (
                  <Table columns={qualityColumns} data={qualityTableData} />
                )}
              </div>
            </>
          )}
        </>
      )}

      {/* Rules Tab */}
      {activeTab === 'rules' && (
        <>
          {rulesError && !rulesLoading && rules.length === 0 ? (
            <div className="flex items-center justify-center py-24">
              <div className="rounded-lg border border-danger-200 bg-danger-50 px-8 py-12 text-center">
                <p className="text-sm text-danger-700">{rulesError}</p>
                <button
                  type="button"
                  onClick={() => void fetchRules(scopeFilter)}
                  className="mt-3 text-sm font-medium text-primary-600 hover:text-primary-700"
                >
                  Retry
                </button>
              </div>
            </div>
          ) : (
            <>
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
                {rulesLoading ? (
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
                  <Table columns={rulesColumns} data={rulesTableData} />
                )}
              </div>
            </>
          )}
        </>
      )}
    </div>
  )
}
