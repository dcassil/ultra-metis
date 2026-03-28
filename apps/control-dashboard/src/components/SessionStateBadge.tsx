type SessionState =
  | 'starting'
  | 'running'
  | 'waiting_for_input'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'stopped'

interface SessionStateBadgeProps {
  state: string
}

const stateConfig: Record<SessionState, { bg: string; dot: string; label: string; pulse?: boolean; icon?: string }> = {
  starting: {
    bg: 'bg-primary-100 text-primary-700',
    dot: 'bg-primary-500',
    label: 'Starting',
    pulse: true,
  },
  running: {
    bg: 'bg-success-100 text-success-700',
    dot: 'bg-success-500',
    label: 'Running',
  },
  waiting_for_input: {
    bg: 'bg-warning-100 text-warning-800 font-semibold',
    dot: 'bg-warning-500',
    label: 'Waiting for Input',
  },
  paused: {
    bg: 'bg-secondary-100 text-secondary-600',
    dot: 'bg-secondary-400',
    label: 'Paused',
  },
  completed: {
    bg: 'bg-success-100 text-success-700',
    dot: 'bg-success-500',
    label: 'Completed',
    icon: 'check',
  },
  failed: {
    bg: 'bg-danger-100 text-danger-700',
    dot: 'bg-danger-500',
    label: 'Failed',
  },
  stopped: {
    bg: 'bg-secondary-100 text-secondary-600',
    dot: 'bg-secondary-400',
    label: 'Stopped',
    icon: 'x',
  },
}

export function SessionStateBadge({ state }: SessionStateBadgeProps) {
  const config = stateConfig[state as SessionState] ?? {
    bg: 'bg-secondary-100 text-secondary-600',
    dot: 'bg-secondary-400',
    label: state,
  }

  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${config.bg}`}>
      <span className={`mr-1.5 h-1.5 w-1.5 rounded-full ${config.dot} ${config.pulse ? 'animate-pulse' : ''}`} />
      {config.icon === 'check' && (
        <svg className="mr-1 h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
        </svg>
      )}
      {config.icon === 'x' && (
        <svg className="mr-1 h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      )}
      {config.label}
    </span>
  )
}
