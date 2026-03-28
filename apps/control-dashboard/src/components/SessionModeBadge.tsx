type SessionMode = 'normal' | 'restricted' | 'elevated'

interface SessionModeBadgeProps {
  mode: string
}

const modeConfig: Record<SessionMode, { bg: string; dot: string; label: string }> = {
  normal: {
    bg: 'bg-success-100 text-success-700',
    dot: 'bg-success-500',
    label: 'Normal',
  },
  restricted: {
    bg: 'bg-warning-100 text-warning-700',
    dot: 'bg-warning-500',
    label: 'Restricted',
  },
  elevated: {
    bg: 'bg-primary-100 text-primary-700',
    dot: 'bg-primary-500',
    label: 'Elevated',
  },
}

export function SessionModeBadge({ mode }: SessionModeBadgeProps) {
  const config = modeConfig[mode as SessionMode] ?? {
    bg: 'bg-secondary-100 text-secondary-600',
    dot: 'bg-secondary-400',
    label: mode,
  }

  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${config.bg}`}>
      <span className={`mr-1.5 h-1.5 w-1.5 rounded-full ${config.dot}`} />
      {config.label}
    </span>
  )
}
