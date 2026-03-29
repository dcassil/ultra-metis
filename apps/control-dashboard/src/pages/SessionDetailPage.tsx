import { useState, useEffect, useCallback, useMemo } from 'react'
import { useParams, Link } from 'react-router-dom'
import { getSession, stopSession, forceStopSession, pauseSession, resumeSession } from '../api/sessions'
import type { SessionResponse } from '../api/sessions'
import { listSessionViolations } from '../api/policies'
import type { PolicyViolationRecord } from '../api/policies'
import { listPendingApprovals } from '../api/interventions'
import type { PendingApproval } from '../api/interventions'
import { getSessionEvents } from '../api/events'
import type { SessionOutputEvent } from '../api/events'
import { Card } from '../components/ui/Card'
import { Button } from '../components/ui/Button'
import { Modal } from '../components/ui/Modal'
import { SessionStateBadge } from '../components/SessionStateBadge'
import { RelativeTime } from '../components/RelativeTime'
import { LiveOutput } from '../components/LiveOutput'
import { SessionTimeline } from '../components/SessionTimeline'
import { ApprovalCard } from '../components/ApprovalCard'
import { GuidanceInput } from '../components/GuidanceInput'
import { ContinueSessionBar } from '../components/ContinueSessionBar'
import { useSessionEventStream } from '../hooks/useSessionEventStream'
import { PlanningContextPanel } from '../components/planning/PlanningContextPanel'
import { MachineLogViewer } from '../components/MachineLogViewer'

const TERMINAL_STATES = ['completed', 'failed', 'stopped']

type TabId = 'overview' | 'live-output' | 'timeline' | 'logs'

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

const TABS: { id: TabId; label: string }[] = [
  { id: 'overview', label: 'Overview' },
  { id: 'live-output', label: 'Live Output' },
  { id: 'timeline', label: 'Timeline' },
  { id: 'logs', label: 'Logs' },
]

export default function SessionDetailPage() {
  const { id } = useParams<{ id: string }>()
  const [session, setSession] = useState<SessionResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  // Tab state — will be set once session loads
  const [activeTab, setActiveTab] = useState<TabId | null>(null)

  // Action loading states
  const [stoppingSession, setStoppingSession] = useState(false)
  const [forceStoppingSession, setForceStoppingSession] = useState(false)
  const [pausingSession, setPausingSession] = useState(false)
  const [resumingSession, setResumingSession] = useState(false)

  // Force stop confirmation modal
  const [showForceStopModal, setShowForceStopModal] = useState(false)

  // Violations
  const [violations, setViolations] = useState<PolicyViolationRecord[]>([])

  // Pending approvals
  const [pendingApprovals, setPendingApprovals] = useState<PendingApproval[]>([])

  // Historical event hydration
  const [historicalEvents, setHistoricalEvents] = useState<SessionOutputEvent[]>([])
  const [historyLoaded, setHistoryLoaded] = useState(false)

  // Elapsed time ticker
  const [, setTick] = useState(0)

  // Compute the max sequence number from historical events for deduplication
  const maxSequence = useMemo(() => {
    if (!historicalEvents || historicalEvents.length === 0) return undefined
    return Math.max(...historicalEvents.map((e) => e.sequence_num))
  }, [historicalEvents])

  // Determine if session is terminal (needed before hook call)
  const isTerminal = session ? TERMINAL_STATES.includes(session.state) : false

  // SSE event stream — seeded with historical events, deduplicated by sequence
  const { events, isConnected, error: streamError } = useSessionEventStream(id ?? '', {
    initialEvents: historyLoaded ? historicalEvents : undefined,
    startAfterSequence: maxSequence,
    enabled: historyLoaded && !isTerminal,
  })

  const fetchViolations = useCallback(async () => {
    if (!id) return
    try {
      const data = await listSessionViolations(id)
      setViolations(data)
    } catch {
      // Non-critical — violations section just won't show
    }
  }, [id])

  const fetchApprovals = useCallback(async () => {
    if (!id) return
    try {
      const data = await listPendingApprovals(id)
      setPendingApprovals(data)
    } catch {
      // Non-critical — approvals section just won't show
    }
  }, [id])

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

  // Fetch historical events on mount (or when session id changes)
  useEffect(() => {
    if (!id) return
    setHistoryLoaded(false)
    getSessionEvents(id)
      .then((resp) => {
        setHistoricalEvents(resp.events ?? [])
        setHistoryLoaded(true)
      })
      .catch(() => {
        // Continue even if history fails — SSE will still work
        setHistoryLoaded(true)
      })
  }, [id])

  // Auto-refresh every 5 seconds
  useEffect(() => {
    void fetchSession()
    void fetchViolations()
    void fetchApprovals()
    const interval = setInterval(() => {
      void fetchSession()
      void fetchViolations()
      void fetchApprovals()
    }, 5_000)
    return () => clearInterval(interval)
  }, [fetchSession, fetchViolations, fetchApprovals])

  // Set default tab based on session state once loaded
  useEffect(() => {
    if (session && activeTab === null) {
      const isTerminal = TERMINAL_STATES.includes(session.state)
      setActiveTab(isTerminal ? 'overview' : 'live-output')
    }
  }, [session, activeTab])

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

  const showStop = session.state === 'running' || session.state === 'waiting_for_input'
  const showForceStop = !isTerminal
  const showPause = session.state === 'running'
  const showResume = session.state === 'paused' || session.state === 'waiting_for_input'

  const currentTab = activeTab ?? 'overview'

  const activePendingApprovals = pendingApprovals.filter((a) => a.status === 'pending')

  return (
    <div className="flex flex-col" style={{ minHeight: 'calc(100vh - 4rem)' }}>
      <div className="flex-1 space-y-6 pb-4">
        <div className="flex items-center gap-4">
          <Link to="/sessions" className="text-sm text-primary-600 hover:text-primary-800">
            &larr; Back to Sessions
          </Link>
        </div>

        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
          <div className="flex items-center gap-3 min-w-0">
            <h1 className="text-lg sm:text-xl font-semibold text-secondary-900 truncate">{session.title}</h1>
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
        <div className="flex flex-col sm:flex-row flex-wrap gap-2 sm:gap-3">
          {showStop && (
            <Button
              variant="secondary"
              size="sm"
              className="w-full sm:w-auto"
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
              className="w-full sm:w-auto"
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
              className="w-full sm:w-auto"
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
              className="w-full sm:w-auto"
              onClick={() => setShowForceStopModal(true)}
            >
              Force Stop
            </Button>
          )}
        </div>

        {/* Pending Approvals */}
        {activePendingApprovals.length > 0 && (
          <div className="rounded-lg border border-warning-300 bg-warning-50 shadow-sm">
            <div className="border-b border-warning-300 px-4 py-3">
              <h3 className="flex items-center gap-2 text-sm font-medium text-warning-800">
                <span className="relative flex h-2.5 w-2.5">
                  <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-warning-400 opacity-75" />
                  <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-warning-500" />
                </span>
                Pending Approvals ({activePendingApprovals.length})
              </h3>
            </div>
            <div className="p-4 space-y-3">
              {activePendingApprovals.map((approval) => (
                <ApprovalCard
                  key={approval.id}
                  approval={approval}
                  sessionId={session.id}
                  disabled={isTerminal}
                  onResponded={() => void fetchApprovals()}
                />
              ))}
            </div>
          </div>
        )}

        {/* Tab Navigation */}
        <div className="-mx-4 sm:mx-0 border-b border-secondary-200 overflow-x-auto">
          <nav className="-mb-px flex gap-0 sm:gap-6 px-4 sm:px-0" aria-label="Tabs">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                type="button"
                onClick={() => setActiveTab(tab.id)}
                className={`whitespace-nowrap border-b-2 px-4 py-3 text-sm font-medium transition-colors sm:px-0 ${
                  currentTab === tab.id
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-secondary-500 hover:border-secondary-300 hover:text-secondary-700'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        {/* Tab Content */}
        {currentTab === 'overview' && (
          <>
            {/* Session Details */}
            <Card title="Session Details">
              <dl className="grid grid-cols-1 gap-x-6 gap-y-4 sm:grid-cols-2 md:grid-cols-3">
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

            {/* Planning Context */}
            {session.work_item_id && (
              <PlanningContextPanel workItemId={session.work_item_id} />
            )}

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

            {/* Policy Violations */}
            {violations.length > 0 && (
              <div className="rounded-lg border border-danger-200 bg-danger-50 shadow-sm">
                <div className="border-b border-danger-200 px-4 py-3">
                  <h3 className="text-sm font-medium text-danger-800">
                    Policy Violations ({violations.length})
                  </h3>
                </div>
                <div className="p-4 space-y-3">
                  {violations.map((v) => (
                    <div
                      key={v.id}
                      className="rounded-md border border-danger-200 bg-white p-3"
                    >
                      <div className="flex items-start justify-between gap-4">
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <code className="rounded bg-danger-100 px-1.5 py-0.5 text-xs font-mono text-danger-700">
                              {v.action}
                            </code>
                            <span className="text-xs text-secondary-500 capitalize">{v.policy_scope}</span>
                          </div>
                          <p className="text-sm text-secondary-700">{v.reason}</p>
                        </div>
                        <span className="shrink-0 text-xs text-secondary-500">
                          <RelativeTime timestamp={v.timestamp} />
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}

        {currentTab === 'live-output' && (
          <LiveOutput events={events} isConnected={isConnected} error={streamError} />
        )}

        {currentTab === 'timeline' && (
          <SessionTimeline events={events} />
        )}

        {currentTab === 'logs' && (
          <MachineLogViewer machineId={session.machine_id} />
        )}
      </div>

      {/* Guidance Input — sticky at the bottom on mobile, visible on all tabs */}
      <div className="sticky bottom-0 z-10 bg-white border-t border-secondary-200 sm:static sm:border-t-0">
        {isTerminal ? (
          <ContinueSessionBar sessionId={session.id} />
        ) : (
          <GuidanceInput sessionId={session.id} />
        )}
      </div>

      {/* Force Stop Confirmation Modal */}
      <Modal
        isOpen={showForceStopModal}
        onClose={() => setShowForceStopModal(false)}
        title="Force Stop Session"
        footer={
          <div className="flex flex-col-reverse sm:flex-row sm:justify-end gap-2 sm:gap-3">
            <Button variant="secondary" size="sm" className="w-full sm:w-auto" onClick={() => setShowForceStopModal(false)}>
              Cancel
            </Button>
            <Button variant="danger" size="sm" className="w-full sm:w-auto" loading={forceStoppingSession} onClick={() => void handleForceStop()}>
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
