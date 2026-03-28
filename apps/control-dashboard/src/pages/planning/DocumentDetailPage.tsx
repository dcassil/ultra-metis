import { useState, useEffect, useCallback } from 'react'
import { useParams, Link } from 'react-router-dom'
import type { DocumentDetail } from '../../api/planning'
import { getDocument } from '../../api/planning'
import { Card } from '../../components/ui/Card'
import { DocumentTypeBadge } from '../../components/planning/DocumentTypeBadge'
import { PhaseBadge } from '../../components/planning/PhaseBadge'
import { ShortCodeLink } from '../../components/planning/ShortCodeLink'

export default function DocumentDetailPage() {
  const { shortCode } = useParams<{ shortCode: string }>()
  const [document, setDocument] = useState<DocumentDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchDocument = useCallback(async () => {
    if (!shortCode) return
    try {
      const data = await getDocument(shortCode)
      setDocument(data)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load document')
    } finally {
      setLoading(false)
    }
  }, [shortCode])

  useEffect(() => {
    void fetchDocument()
  }, [fetchDocument])

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

  if (error && !document) {
    return (
      <div className="flex flex-col items-center justify-center py-24 gap-4">
        <p className="text-danger-600">{error}</p>
        <button
          type="button"
          onClick={() => {
            setLoading(true)
            void fetchDocument()
          }}
          className="text-sm font-medium text-primary-600 hover:text-primary-700"
        >
          Retry
        </button>
      </div>
    )
  }

  if (!document) return null

  return (
    <div className="space-y-6">
      {/* Back navigation */}
      <div className="flex items-center gap-4">
        <Link to="/planning/documents" className="text-sm text-primary-600 hover:text-primary-800">
          &larr; Back to Documents
        </Link>
      </div>

      {/* Title row */}
      <div className="flex items-center gap-3">
        <h1 className="text-xl font-semibold text-secondary-900">{document.title}</h1>
        <DocumentTypeBadge type={document.document_type} />
        <PhaseBadge phase={document.phase} />
      </div>

      {error && (
        <div className="rounded-md bg-danger-50 p-3 text-sm text-danger-700">{error}</div>
      )}

      {/* Metadata panel */}
      <Card title="Details">
        <dl className="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-3">
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Short Code</dt>
            <dd className="mt-1 font-mono text-sm text-secondary-900">{document.short_code}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Type</dt>
            <dd className="mt-1">
              <DocumentTypeBadge type={document.document_type} />
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Phase</dt>
            <dd className="mt-1">
              <PhaseBadge phase={document.phase} />
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Parent</dt>
            <dd className="mt-1 text-sm">
              {document.parent_id ? (
                <ShortCodeLink shortCode={document.parent_id} />
              ) : (
                <span className="text-secondary-400">&mdash;</span>
              )}
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Archived</dt>
            <dd className="mt-1 text-sm text-secondary-900">
              {document.archived ? 'Yes' : 'No'}
            </dd>
          </div>
        </dl>
      </Card>

      {/* Content */}
      <Card title="Content">
        {document.content ? (
          <pre className="whitespace-pre-wrap text-sm text-secondary-800 font-sans leading-relaxed">
            {document.content}
          </pre>
        ) : (
          <p className="text-sm text-secondary-500">No content available.</p>
        )}
      </Card>

      {/* Children */}
      {document.children && document.children.length > 0 && (
        <Card title="Children" subtitle={`${document.children.length} child documents`}>
          <div className="divide-y divide-secondary-100">
            {document.children.map((child) => (
              <div key={child.short_code} className="flex items-center gap-3 py-3 first:pt-0 last:pb-0">
                <ShortCodeLink shortCode={child.short_code} />
                <span className="text-sm text-secondary-900">{child.title}</span>
                <DocumentTypeBadge type={child.document_type} />
                <PhaseBadge phase={child.phase} />
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  )
}
