import type { ReactNode } from 'react'

type BadgeVariant = 'online' | 'offline' | 'pending' | 'error' | 'info'

interface BadgeProps {
  variant: BadgeVariant
  children: ReactNode
}

const variantClasses: Record<BadgeVariant, { bg: string; dot: string }> = {
  online: { bg: 'bg-success-100 text-success-700', dot: 'bg-success-500' },
  offline: { bg: 'bg-secondary-100 text-secondary-600', dot: 'bg-secondary-400' },
  pending: { bg: 'bg-warning-100 text-warning-700', dot: 'bg-warning-500' },
  error: { bg: 'bg-danger-100 text-danger-700', dot: 'bg-danger-500' },
  info: { bg: 'bg-primary-100 text-primary-700', dot: 'bg-primary-500' },
}

export function Badge({ variant, children }: BadgeProps) {
  const styles = variantClasses[variant]
  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${styles.bg}`}>
      <span className={`mr-1.5 h-1.5 w-1.5 rounded-full ${styles.dot}`} />
      {children}
    </span>
  )
}
