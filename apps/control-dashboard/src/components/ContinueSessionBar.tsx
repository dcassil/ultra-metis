import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { continueSession } from '../api/sessions'
import { Button } from './ui/Button'

interface ContinueSessionBarProps {
  sessionId: string
}

export function ContinueSessionBar({ sessionId }: ContinueSessionBarProps) {
  const navigate = useNavigate()
  const [message, setMessage] = useState('')
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const canSend = message.trim().length > 0 && !sending

  const handleSend = async () => {
    if (!canSend) return
    setSending(true)
    setError(null)
    try {
      const resp = await continueSession(sessionId, message.trim())
      navigate(`/sessions/${resp.id}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to continue session')
    } finally {
      setSending(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey && canSend) {
      e.preventDefault()
      void handleSend()
    }
  }

  return (
    <div className="border-t border-secondary-200 bg-white px-4 py-3">
      {error && (
        <p className="mb-2 text-xs text-danger-600">{error}</p>
      )}
      <div className="flex items-center gap-3">
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Send follow-up instructions..."
          className="flex-1 rounded-md border border-secondary-300 bg-white px-3 py-2 text-sm shadow-sm placeholder:text-secondary-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
        />
        <Button
          variant="primary"
          size="sm"
          disabled={!canSend}
          loading={sending}
          onClick={() => void handleSend()}
        >
          Continue Session
        </Button>
      </div>
    </div>
  )
}
