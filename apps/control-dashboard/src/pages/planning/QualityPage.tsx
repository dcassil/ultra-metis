import { useState, useEffect, useCallback, useMemo } from 'react'
import type { PlanningDocument, QualityRecord } from '../../api/planning'
import { listDocuments, getQualityRecords } from '../../api/planning'
import { Table } from '../../components/ui/Table'
import { Card } from '../../components/ui/Card'
import { Badge } from '../../components/ui/Badge'
import { DocumentTypeBadge } from '../../components/planning/DocumentTypeBadge'
import { ShortCodeLink } from '../../components/planning/ShortCodeLink'

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

const statusConfig: Record<string, { variant: 'online' | 'offline' | 'pending' | 'error'; label: string }> = {
  pass: { variant: 'online', label: 'Pass' },
  fail: { variant: 'error', label: 'Fail' },
  pending: { variant: 'pending', label: 'Pending' },
  none: { variant: 'offline', label: 'No Records' },
}

const columns = [
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
      const config = statusConfig[row.overall_status]
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

export default function QualityPage() {
  const [documents, setDocuments] = useState<DocumentWithQuality[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchData = useCallback(async () => {
    try {
      setLoading(true)
      const docs = await listDocuments()

      // Fetch quality records for each document
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
      setError(null)
    } catch {
      setError('Failed to load quality data')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void fetchData()
  }, [fetchData])

  const summary = useMemo(() => {
    const total = documents.length
    const withRecords = documents.filter((d) => d.quality_records.length > 0).length
    const passCount = documents.filter((d) => d.overall_status === 'pass').length
    const failCount = documents.filter((d) => d.overall_status === 'fail').length
    return { total, withRecords, passCount, failCount }
  }, [documents])

  const tableData = useMemo(
    () => documents as (DocumentWithQuality & Record<string, unknown>)[],
    [documents],
  )

  if (error && !loading && documents.length === 0) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold text-secondary-900">Quality</h2>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-8 py-12 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => void fetchData()}
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
        <h2 className="text-2xl font-semibold text-secondary-900">Quality</h2>
      </div>

      {/* Summary Cards */}
      {!loading && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-4">
          <Card>
            <div className="text-center">
              <p className="text-2xl font-bold text-secondary-900">{summary.total}</p>
              <p className="text-sm text-secondary-500">Total Documents</p>
            </div>
          </Card>
          <Card>
            <div className="text-center">
              <p className="text-2xl font-bold text-secondary-900">{summary.withRecords}</p>
              <p className="text-sm text-secondary-500">Quality Records</p>
            </div>
          </Card>
          <Card>
            <div className="text-center">
              <p className="text-2xl font-bold text-success-600">{summary.passCount}</p>
              <p className="text-sm text-secondary-500">Passing</p>
            </div>
          </Card>
          <Card>
            <div className="text-center">
              <p className="text-2xl font-bold text-danger-600">{summary.failCount}</p>
              <p className="text-sm text-secondary-500">Failing</p>
            </div>
          </Card>
        </div>
      )}

      {/* Table */}
      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {loading ? (
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
          <Table columns={columns} data={tableData} />
        )}
      </div>
    </div>
  )
}
