import { useNavigate } from 'react-router-dom'
import { ChevronRightIcon } from '@heroicons/react/20/solid'
import type { HierarchyNode } from '../../api/planning'
import { DocumentTypeBadge } from './DocumentTypeBadge'
import { PhaseBadge } from './PhaseBadge'

interface TreeNodeProps {
  node: HierarchyNode
  expandedNodes: Set<string>
  onToggle: (shortCode: string) => void
}

export function TreeNode({ node, expandedNodes, onToggle }: TreeNodeProps) {
  const navigate = useNavigate()
  const hasChildren = node.children.length > 0
  const isExpanded = expandedNodes.has(node.short_code)

  return (
    <div>
      <div className="group flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-secondary-100">
        {/* Expand/collapse chevron */}
        <button
          type="button"
          onClick={() => hasChildren && onToggle(node.short_code)}
          className={`flex h-5 w-5 shrink-0 items-center justify-center rounded transition-colors ${
            hasChildren
              ? 'text-secondary-500 hover:text-secondary-700 cursor-pointer'
              : 'text-transparent cursor-default'
          }`}
          tabIndex={hasChildren ? 0 : -1}
          aria-label={hasChildren ? (isExpanded ? 'Collapse' : 'Expand') : undefined}
        >
          <ChevronRightIcon
            className={`h-4 w-4 transition-transform duration-150 ${
              isExpanded ? 'rotate-90' : ''
            }`}
          />
        </button>

        {/* Document type badge */}
        <DocumentTypeBadge type={node.document_type} />

        {/* Short code */}
        <span className="shrink-0 font-mono text-xs text-secondary-500">
          {node.short_code}
        </span>

        {/* Title (clickable) */}
        <button
          type="button"
          onClick={() => navigate(`/planning/documents/${node.short_code}`)}
          className="min-w-0 truncate text-sm font-medium text-secondary-900 hover:text-primary-600 hover:underline"
        >
          {node.title}
        </button>

        {/* Phase badge */}
        <div className="ml-auto shrink-0">
          <PhaseBadge phase={node.phase} />
        </div>
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <div className="border-l-2 border-secondary-200 ml-4 pl-3">
          {node.children.map((child) => (
            <TreeNode
              key={child.short_code}
              node={child}
              expandedNodes={expandedNodes}
              onToggle={onToggle}
            />
          ))}
        </div>
      )}
    </div>
  )
}
