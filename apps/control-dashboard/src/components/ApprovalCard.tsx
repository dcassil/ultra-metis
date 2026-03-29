import { useState } from 'react'
import type { PendingApproval } from '../api/interventions'
import { respondToApproval } from '../api/interventions'
import { Button } from './ui/Button'

interface ApprovalCardProps {
  approval: PendingApproval
  sessionId: string
  disabled?: boolean
  onResponded?: () => void
}

export function ApprovalCard({ approval, sessionId, disabled, onResponded }: ApprovalCardProps) {
  const [responding, setResponding] = useState(false)
  const [chosenOption, setChosenOption] = useState<string | null>(
    approval.response_choice ?? null,
  )
  const [error, setError] = useState<string | null>(null)
  const [showNote, setShowNote] = useState(false)
  const [note, setNote] = useState('')

  const isResponded = approval.status !== 'pending' || chosenOption !== null

  let options: string[] = []
  try {
    options = JSON.parse(approval.options) as string[]
  } catch {
    options = []
  }

  const handleRespond = async (choice: string) => {
    setResponding(true)
    setError(null)
    try {
      await respondToApproval(sessionId, approval.id, choice, note || undefined)
      setChosenOption(choice)
      onResponded?.()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to respond')
    } finally {
      setResponding(false)
    }
  }

  return (
    <div
      className={`rounded-lg border p-4 transition-colors ${
        isResponded
          ? 'border-secondary-200 bg-secondary-50'
          : 'border-warning-300 bg-warning-50'
      }`}
    >
      {/* Question */}
      <p className={`text-sm font-medium ${isResponded ? 'text-secondary-500' : 'text-secondary-900'}`}>
        {approval.question}
      </p>

      {/* Context */}
      {approval.context && (
        <p className="mt-1 text-xs text-secondary-500">{approval.context}</p>
      )}

      {/* Responded state */}
      {isResponded && (
        <div className="mt-3 flex items-center gap-2 text-sm text-secondary-600">
          <svg className="h-4 w-4 text-success-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
          </svg>
          <span>
            Responded: <span className="font-medium text-secondary-700">{chosenOption ?? approval.response_choice}</span>
          </span>
          {(approval.response_note || note) && (
            <span className="text-secondary-400">
              &mdash; {approval.response_note || note}
            </span>
          )}
        </div>
      )}

      {/* Action buttons */}
      {!isResponded && (
        <div className="mt-3 space-y-3">
          <div className="flex flex-col sm:flex-row gap-2">
            {options.map((option) => (
              <Button
                key={option}
                variant={option.toLowerCase() === 'deny' || option.toLowerCase() === 'reject' ? 'danger' : 'primary'}
                size="sm"
                className="min-h-[44px] w-full sm:w-auto"
                loading={responding}
                disabled={disabled}
                onClick={() => void handleRespond(option)}
              >
                {option}
              </Button>
            ))}
          </div>

          {/* Optional note */}
          {!showNote ? (
            <button
              type="button"
              onClick={() => setShowNote(true)}
              className="text-xs text-primary-600 hover:text-primary-700"
            >
              + Add note
            </button>
          ) : (
            <textarea
              value={note}
              onChange={(e) => setNote(e.target.value)}
              placeholder="Optional note..."
              rows={2}
              className="w-full rounded-md border border-secondary-300 px-3 py-2 text-sm shadow-sm placeholder:text-secondary-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            />
          )}
        </div>
      )}

      {/* Error */}
      {error && (
        <p className="mt-2 text-xs text-danger-600">{error}</p>
      )}
    </div>
  )
}
