import { useState } from 'react'
import { XMarkIcon } from '@heroicons/react/20/solid'
import { useHealth } from '../contexts/HealthContext'

export default function DisconnectedBanner() {
  const { status } = useHealth()
  const [dismissed, setDismissed] = useState(false)
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000'

  if (status !== 'disconnected' || dismissed) return null

  return (
    <div className="flex items-center justify-between bg-danger-50 px-4 py-2 text-sm text-danger-700">
      <p>
        Unable to reach the Control Service. Check that it is running at{' '}
        <code className="rounded bg-danger-100 px-1 font-mono text-xs">{baseUrl}</code>.
      </p>
      <button
        type="button"
        onClick={() => setDismissed(true)}
        className="ml-4 shrink-0 rounded p-1 hover:bg-danger-100"
      >
        <XMarkIcon className="h-4 w-4" />
      </button>
    </div>
  )
}
