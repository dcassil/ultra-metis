import { useState } from 'react'
import type { Machine } from '../api/machines'
import { approveMachine, revokeMachine } from '../api/machines'
import { Button, Card, Badge, Modal } from './ui'

interface PendingMachineCardProps {
  machine: Machine
  onAction: () => void
}

function formatRelativeTime(dateString: string): string {
  const now = Date.now()
  const then = new Date(dateString).getTime()
  const diffMs = now - then

  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return 'just now'

  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`

  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

export default function PendingMachineCard({ machine, onAction }: PendingMachineCardProps) {
  const [approving, setApproving] = useState(false)
  const [rejecting, setRejecting] = useState(false)
  const [showRejectModal, setShowRejectModal] = useState(false)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)

  async function handleApprove() {
    setApproving(true)
    try {
      await approveMachine(machine.id)
      setSuccessMessage('Machine approved')
      setTimeout(() => {
        setSuccessMessage(null)
        onAction()
      }, 1200)
    } catch {
      setApproving(false)
    }
  }

  async function handleReject() {
    setRejecting(true)
    setShowRejectModal(false)
    try {
      await revokeMachine(machine.id)
      setSuccessMessage('Machine rejected')
      setTimeout(() => {
        setSuccessMessage(null)
        onAction()
      }, 1200)
    } catch {
      setRejecting(false)
    }
  }

  return (
    <>
      <Card className="border-warning-200 bg-warning-50/30">
        <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3 sm:gap-4">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <h4 className="text-sm font-semibold text-secondary-900">{machine.name}</h4>
              <Badge variant="pending">Pending</Badge>
            </div>

            <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-secondary-500">
              <span>Platform: {machine.platform}</span>
              <span>Registered: {formatRelativeTime(machine.created_at)}</span>
              {machine.repos_count > 0 && <span>Repos: {machine.repos_count}</span>}
            </div>

            {successMessage && (
              <p className="mt-2 text-xs font-medium text-success-600">{successMessage}</p>
            )}
          </div>

          <div className="flex flex-col sm:flex-row shrink-0 gap-2">
            <Button
              variant="primary"
              size="sm"
              className="w-full sm:w-auto"
              loading={approving}
              disabled={rejecting || !!successMessage}
              onClick={handleApprove}
            >
              Approve
            </Button>
            <Button
              variant="danger"
              size="sm"
              className="w-full sm:w-auto"
              loading={rejecting}
              disabled={approving || !!successMessage}
              onClick={() => setShowRejectModal(true)}
            >
              Reject
            </Button>
          </div>
        </div>
      </Card>

      <Modal
        isOpen={showRejectModal}
        onClose={() => setShowRejectModal(false)}
        title="Reject Machine"
        footer={
          <div className="flex flex-col-reverse sm:flex-row sm:justify-end gap-2 sm:gap-3">
            <Button variant="secondary" size="sm" onClick={() => setShowRejectModal(false)}>
              Cancel
            </Button>
            <Button variant="danger" size="sm" onClick={handleReject}>
              Reject
            </Button>
          </div>
        }
      >
        <p className="text-sm text-secondary-600">
          Are you sure you want to reject this machine? It will be revoked and will not be able to
          connect.
        </p>
      </Modal>
    </>
  )
}
