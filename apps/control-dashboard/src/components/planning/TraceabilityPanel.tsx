import { useState, useEffect, useCallback } from 'react'
import { getDocument } from '../../api/planning'
import { ShortCodeLink } from './ShortCodeLink'
import { DocumentTypeBadge } from './DocumentTypeBadge'

interface AncestorNode {
  short_code: string
  title: string
  document_type: string
}

interface TraceabilityPanelProps {
  shortCode: string
  parentId: string | null
}

export function TraceabilityPanel({ shortCode, parentId }: TraceabilityPanelProps) {
  const [ancestors, setAncestors] = useState<AncestorNode[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchAncestry = useCallback(async () => {
    if (!parentId) {
      setAncestors([])
      return
    }

    setLoading(true)
    setError(null)

    try {
      const chain: AncestorNode[] = []
      let currentParentId: string | null = parentId

      while (currentParentId) {
        const doc = await getDocument(currentParentId)
        chain.unshift({
          short_code: doc.short_code,
          title: doc.title,
          document_type: doc.document_type,
        })
        currentParentId = doc.parent_id
      }

      setAncestors(chain)
    } catch {
      setError('Failed to load ancestry')
    } finally {
      setLoading(false)
    }
  }, [parentId])

  useEffect(() => {
    void fetchAncestry()
  }, [fetchAncestry])

  if (loading) {
    return (
      <div className="text-sm text-secondary-500">Loading traceability...</div>
    )
  }

  if (error) {
    return (
      <div className="text-sm text-danger-600">{error}</div>
    )
  }

  if (ancestors.length === 0 && !parentId) {
    return (
      <div className="text-sm text-secondary-400">No parent hierarchy (top-level document).</div>
    )
  }

  return (
    <nav aria-label="Document ancestry" className="flex items-center flex-wrap gap-1 text-sm">
      {ancestors.map((ancestor, index) => (
        <span key={ancestor.short_code} className="inline-flex items-center gap-1.5">
          {index > 0 && (
            <span className="text-secondary-400" aria-hidden="true">
              &rsaquo;
            </span>
          )}
          <DocumentTypeBadge type={ancestor.document_type} />
          <ShortCodeLink shortCode={ancestor.short_code} />
          <span className="text-secondary-500 hidden sm:inline" title={ancestor.title}>
            {ancestor.title.length > 30 ? `${ancestor.title.slice(0, 30)}...` : ancestor.title}
          </span>
        </span>
      ))}
      {/* Current document (not clickable) */}
      <span className="inline-flex items-center gap-1.5">
        <span className="text-secondary-400" aria-hidden="true">
          &rsaquo;
        </span>
        <span className="font-mono text-sm font-medium text-secondary-900">{shortCode}</span>
      </span>
    </nav>
  )
}
