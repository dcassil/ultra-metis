import { useState, useEffect, useCallback } from 'react'
import { getDocument } from '../../api/planning'
import type { DocumentDetail } from '../../api/planning'
import { Card } from '../ui/Card'
import { DocumentTypeBadge } from './DocumentTypeBadge'
import { PhaseBadge } from './PhaseBadge'
import { ShortCodeLink } from './ShortCodeLink'

interface AncestorNode {
  short_code: string
  title: string
  document_type: string
  phase: string
}

interface PlanningContextPanelProps {
  workItemId: string
}

export function PlanningContextPanel({ workItemId }: PlanningContextPanelProps) {
  const [chain, setChain] = useState<AncestorNode[]>([])
  const [workItem, setWorkItem] = useState<DocumentDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [collapsed, setCollapsed] = useState(() => {
    try {
      return localStorage.getItem('cadre-planning-context-collapsed') === 'true'
    } catch {
      return true
    }
  })

  const fetchContext = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)

      // Fetch the work item
      const doc = await getDocument(workItemId)
      setWorkItem(doc)

      // Walk up the parent chain
      const ancestors: AncestorNode[] = []
      let currentParentId = doc.parent_id

      while (currentParentId) {
        const parent = await getDocument(currentParentId)
        ancestors.unshift({
          short_code: parent.short_code,
          title: parent.title,
          document_type: parent.document_type,
          phase: parent.phase,
        })
        currentParentId = parent.parent_id
      }

      setChain(ancestors)
    } catch {
      setError('Failed to load planning context')
    } finally {
      setLoading(false)
    }
  }, [workItemId])

  useEffect(() => {
    void fetchContext()
  }, [fetchContext])

  const toggleCollapsed = () => {
    setCollapsed((prev) => {
      const next = !prev
      try {
        localStorage.setItem('cadre-planning-context-collapsed', String(next))
      } catch {
        // ignore
      }
      return next
    })
  }

  return (
    <Card>
      <button
        type="button"
        onClick={toggleCollapsed}
        className="flex w-full items-center justify-between text-left"
      >
        <h3 className="text-sm font-medium text-secondary-900">Planning Context</h3>
        <svg
          className={`h-5 w-5 text-secondary-400 transition-transform ${collapsed ? '' : 'rotate-180'}`}
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path fillRule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clipRule="evenodd" />
        </svg>
      </button>

      {!collapsed && (
        <div className="mt-3 space-y-3">
          {loading && (
            <div className="text-sm text-secondary-500">Loading planning context...</div>
          )}

          {error && (
            <div className="text-sm text-danger-600">{error}</div>
          )}

          {!loading && !error && (
            <>
              {/* Ancestry breadcrumb */}
              <nav aria-label="Work item ancestry" className="space-y-2">
                {chain.map((ancestor, index) => (
                  <div
                    key={ancestor.short_code}
                    className="flex items-center gap-2"
                    style={{ paddingLeft: `${index * 16}px` }}
                  >
                    {index > 0 && (
                      <span className="text-secondary-300" aria-hidden="true">
                        &lfloor;
                      </span>
                    )}
                    <DocumentTypeBadge type={ancestor.document_type} />
                    <ShortCodeLink shortCode={ancestor.short_code} />
                    <span className="truncate text-sm text-secondary-600">{ancestor.title}</span>
                    <PhaseBadge phase={ancestor.phase} />
                  </div>
                ))}

                {/* Current work item */}
                {workItem && (
                  <div
                    className="flex items-center gap-2 rounded-md bg-primary-50 px-2 py-1"
                    style={{ paddingLeft: `${chain.length * 16}px` }}
                  >
                    {chain.length > 0 && (
                      <span className="text-secondary-300" aria-hidden="true">
                        &lfloor;
                      </span>
                    )}
                    <DocumentTypeBadge type={workItem.document_type} />
                    <ShortCodeLink shortCode={workItem.short_code} />
                    <span className="truncate text-sm font-medium text-secondary-900">{workItem.title}</span>
                    <PhaseBadge phase={workItem.phase} />
                  </div>
                )}
              </nav>
            </>
          )}
        </div>
      )}
    </Card>
  )
}
