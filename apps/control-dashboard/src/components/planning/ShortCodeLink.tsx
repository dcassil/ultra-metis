import { Link } from 'react-router-dom'

interface ShortCodeLinkProps {
  shortCode: string
  className?: string
}

export function ShortCodeLink({ shortCode, className = '' }: ShortCodeLinkProps) {
  return (
    <Link
      to={`/planning/documents/${shortCode}`}
      className={`font-mono text-sm text-primary-400 hover:text-primary-300 hover:underline ${className}`}
    >
      {shortCode}
    </Link>
  )
}
