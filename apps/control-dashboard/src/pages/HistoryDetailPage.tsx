import { useState, useEffect, useCallback } from 'react'
import { useParams, Link } from 'react-router-dom'
import { getHistorySession, getSessionOutcome } from '../api/history'
import type { SessionOutcome } from '../api/history'
import type { SessionResponse } from '../api/sessions'
import { getSessionEvents } from '../api/events'
import type { SessionOutputEvent } from '../api/events'
import { Card } from '../components/ui/Card'
import { Badge } from '../components/ui/Badge'
import { Button } from '../components/ui/Button'
import { SessionTimeline } from '../components/SessionTimeline'
import { RelativeTime } from '../components/RelativeTime'

type TabId = 'timeline' | 'all-events'

const TABS: { id: TabId; label: string }[] = [
  { id: 'timeline', label: 'Timeline' },
  { id: 'all-events', label: 'All Events' },
]

function outcomeStatusVariant(status: string): 'online' | 'error' | 'pending' {
  switch (status) {
    case 'success':
      return 'online'
    case 'failure':
      return 'error'
    case 'partial':
      return 'pending'
    default:
      return 'pending'
  }
}

function outcomeStatusLabel(status: string): string {
  switch (status) {
    case 'success':
      return 'Success'
    case 'failure':
      return 'Failure'
    case 'partial':
      return 'Partial'
    default:
      return status
  }
}

function formatDuration(seconds: number): string {
  if (seconds < 0) return '\u2014'
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  if (hours > 0) return `${hours}h ${minutes}m ${secs}s`
  if (minutes > 0) return `${minutes}m ${secs}s`
  return `${secs}s`
}

function formatEventTime(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return ''
  }
}

function eventCategoryColor(category: string | null): string {
  switch (category) {
    case 'error':
      return 'text-danger-600'
    case 'warning':
      return 'text-warning-700'
    case 'summary':
      return 'text-primary-700'
    default:
      return 'text-secondary-700'
  }
}

export default function HistoryDetailPage() {
  const { id } = useParams<{ id: string }>()
  const [session, setSession] = useState<SessionResponse | null>(null)
  const [outcome, setOutcome] = useState<SessionOutcome | null>(null)
  const [events, setEvents] = useState<SessionOutputEvent[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<TabId>('timeline')

  const fetchData = useCallback(async () => {
    if (!id) return
    try {
      const [sessionData, outcomeData, eventsData] = await Promise.all([
        getHistorySession(id),
        getSessionOutcome(id).catch(() => null),
        getSessionEvents(id),
      ])
      setSession(sessionData)
      setOutcome(outcomeData)
      setEvents(eventsData.events)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load session history')
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    void fetchData()
  }, [fetchData])

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
        <Button variant="secondary" onClick={() => { setLoading(true); void fetchData() }}>
          Retry
        </Button>
      </div>
    )
  }

  if (!session) return null

  return (
    <div className="space-y-6">
      {/* Back navigation */}
      <div className="flex items-center gap-4">
        <Link to="/history" className="text-sm text-primary-600 hover:text-primary-800">
          &larr; Back to History
        </Link>
      </div>

      {/* Header */}
      <div className="flex items-center gap-3">
        <h1 className="text-xl font-semibold text-secondary-900">{session.title}</h1>
      </div>

      {error && (
        <div className="rounded-md bg-warning-50 border border-warning-200 p-3 text-sm text-warning-700">
          {error} (showing cached data)
        </div>
      )}

      {/* Outcome Card */}
      {outcome && (
        <Card title="Session Outcome">
          <div className="space-y-4">
            {/* Status badge and summary */}
            <div className="flex items-start gap-3">
              <Badge variant={outcomeStatusVariant(outcome.status)}>
                {outcomeStatusLabel(outcome.status)}
              </Badge>
            </div>

            {outcome.summary && (
              <p className="text-sm text-secondary-700">{outcome.summary}</p>
            )}

            {/* Stats row */}
            <dl className="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-4">
              <div>
                <dt className="text-xs font-medium uppercase text-secondary-500">Duration</dt>
                <dd className="mt-1 text-sm font-mono text-secondary-900">
                  {formatDuration(outcome.duration_seconds)}
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium uppercase text-secondary-500">Event Count</dt>
                <dd className="mt-1 text-sm font-mono text-secondary-900">
                  {outcome.event_count}
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium uppercase text-secondary-500">Interventions</dt>
                <dd className="mt-1 text-sm font-mono text-secondary-900">
                  {outcome.intervention_count}
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium uppercase text-secondary-500">Created</dt>
                <dd className="mt-1 text-sm text-secondary-900">
                  <RelativeTime timestamp={outcome.created_at} />
                </dd>
              </div>
            </dl>
          </div>
        </Card>
      )}

      {/* No outcome fallback */}
      {!outcome && (
        <Card title="Session Outcome">
          <p className="text-sm text-secondary-500">No outcome data available for this session.</p>
        </Card>
      )}

      {/* Tab Navigation */}
      <div className="border-b border-secondary-200">
        <nav className="-mb-px flex gap-6" aria-label="Tabs">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              className={`whitespace-nowrap border-b-2 py-2.5 text-sm font-medium transition-colors ${
                activeTab === tab.id
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
      {activeTab === 'timeline' && (
        <SessionTimeline events={events} />
      )}

      {activeTab === 'all-events' && (
        <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
          {events.length === 0 ? (
            <div className="p-8 text-center">
              <p className="text-sm text-secondary-500">No events recorded for this session.</p>
            </div>
          ) : (
            <div className="max-h-[600px] overflow-y-auto">
              <ul className="divide-y divide-secondary-100">
                {events.map((event) => (
                  <li key={event.id} className="px-4 py-3 hover:bg-secondary-50">
                    <div className="flex items-start justify-between gap-4">
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <code className="rounded bg-secondary-100 px-1.5 py-0.5 text-xs font-mono text-secondary-600">
                            {event.event_type}
                          </code>
                          {event.category && (
                            <span className={`text-xs font-medium capitalize ${eventCategoryColor(event.category)}`}>
                              {event.category}
                            </span>
                          )}
                          <span className="text-xs text-secondary-400">#{event.sequence_num}</span>
                        </div>
                        <p className={`text-sm whitespace-pre-wrap break-words ${eventCategoryColor(event.category)}`}>
                          {event.content}
                        </p>
                      </div>
                      <span className="shrink-0 text-xs text-secondary-500 font-mono">
                        {formatEventTime(event.timestamp)}
                      </span>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
