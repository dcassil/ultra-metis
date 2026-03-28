import { useState, useEffect, useCallback } from 'react'
import { useParams, Link } from 'react-router-dom'
import { getSession, stopSession, forceStopSession, pauseSession, resumeSession } from '../api/sessions'
import type { SessionResponse } from '../api/sessions'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Modal } from '../components/ui/Modal'
import { SessionStateBadge } from '../components/SessionStateBadge'
import { RelativeTime } from '../components/RelativeTime'

const TERMINAL_STATES = ['completed', 'failed', 'stopped']

function formatElapsed(startedAt: string | undefined, completedAt: string | undefined): string {
  if (!startedAt) return '\u2014'
  const end = completedAt ? Date.parse(completedAt) : Date.now()
  const diffMs = end - Date.parse(startedAt)
  if (diffMs < 0) return '\u2014'
  const totalSeconds = Math.floor(diffMs / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  if (hours > 0) return `${hours}h ${minutes}m ${seconds}s`
  if (minutes > 0) return `${minutes}m ${seconds}s`
  return `${seconds}s`
}

export default function SessionDetailPage() {
  const { id } = useParams<{ id: string }>()
  const [session, setSession] = useState<SessionResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  // Action loading states
  const [stoppingSession, setStoppingSession] = useState(false)
  const [forceStoppingSession, setForceStoppingSession] = useState(false)
  const [pausingSession, setPausingSession] = useState(false)
  const [resumingSession, setResumingSession] = useState(false)

  // Force stop confirmation modal
  const [showForceStopModal, setShowForceStopModal] = useState(false)

  // Elapsed time ticker
  const [, setTick] = useState(0)

  const fetchSession = useCallback(async () => {
    if (!id) return
    try {
      const data = await getSession(id)
      setSession(data)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load session')
    } finally {
      setLoading(false)
    }
  }, [id])

  // Auto-refresh every 5 seconds
  useEffect(() => {
    void fetchSession()
    const interval = setInterval(() => void fetchSession(), 5_000)
    return () => clearInterval(interval)
  }, [fetchSession])

  // Elapsed time counter - tick every second when session is active
  useEffect(() => {
    if (!session?.started_at || TERMINAL_STATES.includes(session.state)) return
    const interval = setInterval(() => setTick((t) => t + 1), 1_000)
    return () => clearInterval(interval)
  }, [session?.started_at, session?.state])

  const handleStop = async () => {
    if (!id) return
    setStoppingSession(true)
    setActionError(null)
    try {
      await stopSession(id)
      await fetchSession()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Failed to stop session')
    } finally {
      setStoppingSession(false)
    }
  }

  const handleForceStop = async () => {
    if (!id) return
    setForceStoppingSession(true)
    setActionError(null)
    try {
      await forceStopSession(id)
      setShowForceStopModal(false)
      await fetchSession()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Failed to force stop session')
      setShowForceStopModal(false)
    } finally {
      setForceStoppingSession(false)
    }
  }

  const handlePause = async () => {
    if (!id) return
    setPausingSession(true)
    setActionError(null)
    try {
      await pauseSession(id)
      await fetchSession()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Failed to pause session')
    } finally {
      setPausingSession(false)
    }
  }

  const handleResume = async () => {
    if (!id) return
    setResumingSession(true)
    setActionError(null)
    try {
      await resumeSession(id)
      await fetchSession()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Failed to resume session')
    } finally {
      setResumingSession(false)
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-24">
        <svg className="h-8 w-8 animate-spin text-primary-600" viewBox="0 0 24 24" fill="none">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
      </div>
    )
  }

  if (error && !session) {
    return (
      <div className="flex flex-col items-center justify-center py-24 gap-4">
        <p className="text-danger-600">{error}</p>
        <Button variant="secondary" onClick={() => { setLoading(true); void fetchSession() }}>
          Retry
        </Button>
      </div>
    )
  }

  if (!session) return null

  const isTerminal = TERMINAL_STATES.includes(session.state)
  const showStop = session.state === 'running' || session.state === 'waiting_for_input'
  const showForceStop = !isTerminal
  const showPause = session.state === 'running'
  const showResume = session.state === 'paused' || session.state === 'waiting_for_input'

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Link to="/sessions" className="text-sm text-primary-600 hover:text-primary-800">
          &larr; Back to Sessions
        </Link>
      </div>

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-semibold text-secondary-900">{session.title}</h1>
          <SessionStateBadge state={session.state} />
        </div>
      </div>

      {actionError && (
        <div className="rounded-md bg-danger-50 border border-danger-200 p-3 text-sm text-danger-700">
          {actionError}
        </div>
      )}

      {error && (
        <div className="rounded-md bg-warning-50 border border-warning-200 p-3 text-sm text-warning-700">
          {error} (showing cached data)
        </div>
      )}

      {/* Control Actions */}
      <div className="flex flex-wrap gap-3">
        {showStop && (
          <Button
            variant="secondary"
            size="sm"
            loading={stoppingSession}
            onClick={() => void handleStop()}
          >
            Stop
          </Button>
        )}
        {showPause && (
          <Button
            variant="secondary"
            size="sm"
            loading={pausingSession}
            onClick={() => void handlePause()}
          >
            Pause
          </Button>
        )}
        {showResume && (
          <Button
            variant="primary"
            size="sm"
            loading={resumingSession}
            onClick={() => void handleResume()}
          >
            Resume
          </Button>
        )}
        {showForceStop && (
          <Button
            variant="danger"
            size="sm"
            onClick={() => setShowForceStopModal(true)}
          >
            Force Stop
          </Button>
        )}
      </div>

      {/* Session Details */}
      <Card title="Session Details">
        <dl className="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-3">
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Machine</dt>
            <dd className="mt-1 text-sm text-secondary-900">{session.machine_id}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Repository</dt>
            <dd className="mt-1 text-sm text-secondary-900">{session.repo_path}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Autonomy Level</dt>
            <dd className="mt-1 text-sm capitalize text-secondary-900">{session.autonomy_level}</dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Elapsed Time</dt>
            <dd className="mt-1 text-sm font-mono text-secondary-900">
              {formatElapsed(session.started_at, session.completed_at)}
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Created</dt>
            <dd className="mt-1 text-sm text-secondary-900">
              <RelativeTime timestamp={session.created_at} />
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase text-secondary-500">Last Updated</dt>
            <dd className="mt-1 text-sm text-secondary-900">
              <RelativeTime timestamp={session.updated_at} />
            </dd>
          </div>
          {session.work_item_id && (
            <div>
              <dt className="text-xs font-medium uppercase text-secondary-500">Work Item</dt>
              <dd className="mt-1 text-sm text-secondary-900">{session.work_item_id}</dd>
            </div>
          )}
        </dl>
      </Card>

      {/* Instructions */}
      <Card title="Instructions">
        <pre className="overflow-x-auto whitespace-pre-wrap rounded-md bg-secondary-50 p-3 text-sm text-secondary-800">
          {session.instructions}
        </pre>
      </Card>

      {/* Context (if present) */}
      {session.context && (
        <Card title="Context">
          <pre className="overflow-x-auto whitespace-pre-wrap rounded-md bg-secondary-50 p-3 text-sm text-secondary-800">
            {session.context}
          </pre>
        </Card>
      )}

      {/* Force Stop Confirmation Modal */}
      <Modal
        isOpen={showForceStopModal}
        onClose={() => setShowForceStopModal(false)}
        title="Force Stop Session"
        footer={
          <div className="flex justify-end gap-3">
            <Button variant="secondary" size="sm" onClick={() => setShowForceStopModal(false)}>
              Cancel
            </Button>
            <Button variant="danger" size="sm" loading={forceStoppingSession} onClick={() => void handleForceStop()}>
              Force Stop
            </Button>
          </div>
        }
      >
        <p className="text-sm text-secondary-600">
          Are you sure you want to force stop this session? This will immediately terminate the session
          without waiting for a graceful shutdown. Any in-progress work may be lost.
        </p>
      </Modal>
    </div>
  )
}
