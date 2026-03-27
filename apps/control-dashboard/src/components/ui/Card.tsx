import type { ReactNode } from 'react'

interface CardProps {
  title?: string
  subtitle?: string
  footer?: ReactNode
  children: ReactNode
  className?: string
}

export function Card({ title, subtitle, footer, children, className = '' }: CardProps) {
  return (
    <div className={`rounded-lg border border-secondary-200 bg-white shadow-sm ${className}`}>
      {(title || subtitle) && (
        <div className="border-b border-secondary-200 px-4 py-3">
          {title && <h3 className="text-sm font-medium text-secondary-900">{title}</h3>}
          {subtitle && <p className="mt-1 text-sm text-secondary-500">{subtitle}</p>}
        </div>
      )}
      <div className="p-4">{children}</div>
      {footer && <div className="border-t border-secondary-200 px-4 py-3">{footer}</div>}
    </div>
  )
}
