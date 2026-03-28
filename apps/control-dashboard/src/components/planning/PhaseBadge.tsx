const phaseConfig: Record<string, { bg: string; dot: string; label: string }> = {
  // Vision phases
  draft: { bg: 'bg-secondary-100 text-secondary-600', dot: 'bg-secondary-400', label: 'Draft' },
  review: { bg: 'bg-warning-100 text-warning-700', dot: 'bg-warning-500', label: 'Review' },
  published: { bg: 'bg-success-100 text-success-700', dot: 'bg-success-500', label: 'Published' },

  // Strategy phases
  shaping: { bg: 'bg-secondary-100 text-secondary-600', dot: 'bg-secondary-400', label: 'Shaping' },

  // Initiative phases
  discovery: { bg: 'bg-purple-100 text-purple-700', dot: 'bg-purple-500', label: 'Discovery' },
  design: { bg: 'bg-blue-100 text-blue-700', dot: 'bg-blue-500', label: 'Design' },
  ready: { bg: 'bg-teal-100 text-teal-700', dot: 'bg-teal-500', label: 'Ready' },
  decompose: { bg: 'bg-indigo-100 text-indigo-700', dot: 'bg-indigo-500', label: 'Decompose' },

  // Shared phases
  active: { bg: 'bg-primary-100 text-primary-700', dot: 'bg-primary-500', label: 'Active' },
  completed: { bg: 'bg-success-100 text-success-700', dot: 'bg-success-500', label: 'Completed' },

  // Task phases
  backlog: { bg: 'bg-secondary-100 text-secondary-600', dot: 'bg-secondary-400', label: 'Backlog' },
  todo: { bg: 'bg-secondary-100 text-secondary-600', dot: 'bg-secondary-400', label: 'To Do' },
  blocked: { bg: 'bg-danger-100 text-danger-700', dot: 'bg-danger-500', label: 'Blocked' },

  // ADR phases
  discussion: { bg: 'bg-warning-100 text-warning-700', dot: 'bg-warning-500', label: 'Discussion' },
  decided: { bg: 'bg-success-100 text-success-700', dot: 'bg-success-500', label: 'Decided' },
  superseded: { bg: 'bg-secondary-100 text-secondary-600', dot: 'bg-secondary-400', label: 'Superseded' },
}

export function PhaseBadge({ phase }: { phase: string }) {
  const config = phaseConfig[phase] ?? {
    bg: 'bg-secondary-100 text-secondary-600',
    dot: 'bg-secondary-400',
    label: phase,
  }

  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${config.bg}`}>
      <span className={`mr-1.5 h-1.5 w-1.5 rounded-full ${config.dot}`} />
      {config.label}
    </span>
  )
}
