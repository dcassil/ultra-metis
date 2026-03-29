import { useState, useEffect, useCallback, useMemo } from 'react'
import { getHierarchy } from '../../api/planning'
import type { HierarchyNode } from '../../api/planning'
import { TreeNode } from '../../components/planning/TreeNode'

const STORAGE_KEY = 'cadre-hierarchy-expanded'

function loadExpanded(): Set<string> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const arr = JSON.parse(raw)
      if (Array.isArray(arr)) return new Set(arr as string[])
    }
  } catch {
    // ignore corrupt data
  }
  return new Set()
}

function saveExpanded(expanded: Set<string>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...expanded]))
  } catch {
    // ignore quota errors
  }
}

/** Collect all short_codes from a hierarchy tree. */
function collectAllCodes(nodes: HierarchyNode[]): string[] {
  const codes: string[] = []
  function walk(node: HierarchyNode) {
    if (node.children.length > 0) {
      codes.push(node.short_code)
      node.children.forEach(walk)
    }
  }
  nodes.forEach(walk)
  return codes
}

/** Count nodes by document_type across the entire tree. */
function countByType(nodes: HierarchyNode[]): Record<string, number> {
  const counts: Record<string, number> = {}
  function walk(node: HierarchyNode) {
    counts[node.document_type] = (counts[node.document_type] ?? 0) + 1
    node.children.forEach(walk)
  }
  nodes.forEach(walk)
  return counts
}

export default function HierarchyPage() {
  const [nodes, setNodes] = useState<HierarchyNode[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(loadExpanded)

  const fetchHierarchy = useCallback(async () => {
    try {
      const data = await getHierarchy()
      setNodes(data)
      setError(null)
    } catch {
      setError('Failed to load hierarchy')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchHierarchy()
  }, [fetchHierarchy])

  function handleToggle(shortCode: string) {
    setExpandedNodes((prev) => {
      const next = new Set(prev)
      if (next.has(shortCode)) {
        next.delete(shortCode)
      } else {
        next.add(shortCode)
      }
      saveExpanded(next)
      return next
    })
  }

  function handleExpandAll() {
    const allCodes = collectAllCodes(nodes)
    const next = new Set(allCodes)
    saveExpanded(next)
    setExpandedNodes(next)
  }

  function handleCollapseAll() {
    const next = new Set<string>()
    saveExpanded(next)
    setExpandedNodes(next)
  }

  const typeCounts = useMemo(() => countByType(nodes), [nodes])

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-semibold text-secondary-900">Hierarchy</h2>
        </div>
        <div className="rounded-lg border border-secondary-200 bg-white shadow-sm p-4 space-y-3">
          {/* Skeleton rows */}
          {[1, 2, 3, 4, 5].map((i) => (
            <div key={i} className="flex items-center gap-3 animate-pulse">
              <div className="h-5 w-5 rounded bg-secondary-200" />
              <div className="h-5 w-16 rounded-full bg-secondary-200" />
              <div className="h-4 w-20 rounded bg-secondary-200" />
              <div className="h-4 rounded bg-secondary-200" style={{ width: `${120 + i * 30}px` }} />
              <div className="ml-auto h-5 w-16 rounded-full bg-secondary-200" />
            </div>
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-semibold text-secondary-900">Hierarchy</h2>
        </div>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-4 py-12 sm:px-8 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => {
                setLoading(true)
                fetchHierarchy()
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
        <h2 className="text-2xl font-semibold text-secondary-900">Hierarchy</h2>
        {nodes.length > 0 && (
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handleExpandAll}
              className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm font-medium text-secondary-700 shadow-sm hover:bg-secondary-50"
            >
              Expand All
            </button>
            <button
              type="button"
              onClick={handleCollapseAll}
              className="rounded-md border border-secondary-300 bg-white px-3 py-1.5 text-sm font-medium text-secondary-700 shadow-sm hover:bg-secondary-50"
            >
              Collapse All
            </button>
          </div>
        )}
      </div>

      {/* Summary counts */}
      {nodes.length > 0 && (
        <p className="text-sm text-secondary-600">
          {[
            typeCounts['vision'] ? `${typeCounts['vision']} vision${typeCounts['vision'] === 1 ? '' : 's'}` : null,
            typeCounts['strategy'] ? `${typeCounts['strategy']} strateg${typeCounts['strategy'] === 1 ? 'y' : 'ies'}` : null,
            typeCounts['initiative'] ? `${typeCounts['initiative']} initiative${typeCounts['initiative'] === 1 ? '' : 's'}` : null,
            typeCounts['task'] ? `${typeCounts['task']} task${typeCounts['task'] === 1 ? '' : 's'}` : null,
            typeCounts['adr'] ? `${typeCounts['adr']} ADR${typeCounts['adr'] === 1 ? '' : 's'}` : null,
          ]
            .filter(Boolean)
            .join(', ')}
        </p>
      )}

      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {nodes.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">No planning documents found.</p>
          </div>
        ) : (
          <div className="p-3 sm:p-4 space-y-1 overflow-x-auto">
            {nodes.map((node) => (
              <TreeNode
                key={node.short_code}
                node={node}
                expandedNodes={expandedNodes}
                onToggle={handleToggle}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
