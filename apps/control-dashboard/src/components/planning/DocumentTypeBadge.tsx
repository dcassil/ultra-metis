const typeConfig: Record<string, { bg: string; dot: string; label: string }> = {
  vision: {
    bg: 'bg-purple-100 text-purple-700',
    dot: 'bg-purple-500',
    label: 'Vision',
  },
  strategy: {
    bg: 'bg-blue-100 text-blue-700',
    dot: 'bg-blue-500',
    label: 'Strategy',
  },
  initiative: {
    bg: 'bg-teal-100 text-teal-700',
    dot: 'bg-teal-500',
    label: 'Initiative',
  },
  task: {
    bg: 'bg-secondary-100 text-secondary-600',
    dot: 'bg-secondary-400',
    label: 'Task',
  },
  adr: {
    bg: 'bg-amber-100 text-amber-700',
    dot: 'bg-amber-500',
    label: 'ADR',
  },
  epic: {
    bg: 'bg-indigo-100 text-indigo-700',
    dot: 'bg-indigo-500',
    label: 'Epic',
  },
  story: {
    bg: 'bg-cyan-100 text-cyan-700',
    dot: 'bg-cyan-500',
    label: 'Story',
  },
  product_doc: {
    bg: 'bg-purple-100 text-purple-700',
    dot: 'bg-purple-500',
    label: 'Product Doc',
  },
}

export function DocumentTypeBadge({ type: docType }: { type: string }) {
  const config = typeConfig[docType] ?? {
    bg: 'bg-secondary-100 text-secondary-600',
    dot: 'bg-secondary-400',
    label: docType,
  }

  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${config.bg}`}>
      <span className={`mr-1.5 h-1.5 w-1.5 rounded-full ${config.dot}`} />
      {config.label}
    </span>
  )
}
