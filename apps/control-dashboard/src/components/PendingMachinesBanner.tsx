import { ExclamationTriangleIcon } from '@heroicons/react/24/outline'
import { Button } from './ui'

interface PendingMachinesBannerProps {
  count: number
  onViewPending: () => void
}

export default function PendingMachinesBanner({ count, onViewPending }: PendingMachinesBannerProps) {
  if (count <= 0) return null

  return (
    <div className="mb-4 flex items-center justify-between rounded-lg border border-warning-200 bg-warning-50 px-4 py-3">
      <div className="flex items-center gap-3">
        <ExclamationTriangleIcon className="h-5 w-5 text-warning-600" aria-hidden="true" />
        <p className="text-sm font-medium text-warning-800">
          {count} machine{count !== 1 ? 's' : ''} awaiting approval
        </p>
      </div>
      <Button variant="secondary" size="sm" onClick={onViewPending}>
        Review
      </Button>
    </div>
  )
}
