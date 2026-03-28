import { useState } from 'react'
import { injectGuidance } from '../api/interventions'
import { Button } from './ui/Button'
import { Modal } from './ui/Modal'

type InjectionType = 'normal' | 'side_note' | 'interrupt'

const INJECTION_TYPES: { value: InjectionType; label: string }[] = [
  { value: 'normal', label: 'Normal' },
  { value: 'side_note', label: 'Side Note' },
  { value: 'interrupt', label: 'Interrupt' },
]

interface GuidanceInputProps {
  sessionId: string
  disabled?: boolean
}

export function GuidanceInput({ sessionId, disabled }: GuidanceInputProps) {
  const [message, setMessage] = useState('')
  const [injectionType, setInjectionType] = useState<InjectionType>('normal')
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showSuccess, setShowSuccess] = useState(false)
  const [showInterruptConfirm, setShowInterruptConfirm] = useState(false)

  const canSend = message.trim().length > 0 && !disabled && !sending

  const doSend = async () => {
    if (!canSend) return
    setSending(true)
    setError(null)
    try {
      await injectGuidance(sessionId, message.trim(), injectionType)
      setMessage('')
      setShowSuccess(true)
      setTimeout(() => setShowSuccess(false), 2_000)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to inject guidance')
    } finally {
      setSending(false)
    }
  }

  const handleSend = () => {
    if (injectionType === 'interrupt') {
      setShowInterruptConfirm(true)
    } else {
      void doSend()
    }
  }

  const handleConfirmInterrupt = () => {
    setShowInterruptConfirm(false)
    void doSend()
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey && canSend) {
      e.preventDefault()
      handleSend()
    }
  }

  return (
    <>
      <div className="border-t border-secondary-200 bg-white px-4 py-3">
        {error && (
          <p className="mb-2 text-xs text-danger-600">{error}</p>
        )}
        <div className="flex items-center gap-3">
          {/* Injection type selector */}
          <div className="flex rounded-md border border-secondary-300 bg-secondary-50 text-xs">
            {INJECTION_TYPES.map((type) => (
              <button
                key={type.value}
                type="button"
                disabled={disabled}
                onClick={() => setInjectionType(type.value)}
                className={`px-2.5 py-1.5 font-medium transition-colors first:rounded-l-md last:rounded-r-md disabled:opacity-50 ${
                  injectionType === type.value
                    ? type.value === 'interrupt'
                      ? 'bg-danger-600 text-white'
                      : 'bg-primary-600 text-white'
                    : 'text-secondary-600 hover:bg-secondary-100'
                }`}
              >
                {type.label}
              </button>
            ))}
          </div>

          {/* Message input */}
          <input
            type="text"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={disabled}
            placeholder={disabled ? 'Session is terminal' : 'Inject guidance...'}
            className="flex-1 rounded-md border border-secondary-300 bg-white px-3 py-2 text-sm shadow-sm placeholder:text-secondary-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 disabled:bg-secondary-50 disabled:text-secondary-400"
          />

          {/* Send button */}
          <Button
            variant={injectionType === 'interrupt' ? 'danger' : 'primary'}
            size="sm"
            disabled={!canSend}
            loading={sending}
            onClick={handleSend}
          >
            {showSuccess ? (
              <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
            ) : (
              'Send'
            )}
          </Button>
        </div>
      </div>

      {/* Interrupt confirmation modal */}
      <Modal
        isOpen={showInterruptConfirm}
        onClose={() => setShowInterruptConfirm(false)}
        title="Confirm Interrupt"
        footer={
          <div className="flex justify-end gap-3">
            <Button variant="secondary" size="sm" onClick={() => setShowInterruptConfirm(false)}>
              Cancel
            </Button>
            <Button variant="danger" size="sm" loading={sending} onClick={handleConfirmInterrupt}>
              Send Interrupt
            </Button>
          </div>
        }
      >
        <p className="text-sm text-secondary-600">
          Interrupts will forcefully inject guidance into the session, potentially disrupting the
          current operation. Are you sure you want to send this interrupt?
        </p>
        <div className="mt-3 rounded-md bg-secondary-50 p-3">
          <p className="text-sm text-secondary-700">{message}</p>
        </div>
      </Modal>
    </>
  )
}
