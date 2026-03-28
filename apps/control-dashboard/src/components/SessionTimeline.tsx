import type { SessionOutputEvent } from '../api/events'

interface SessionTimelineProps {
  events: SessionOutputEvent[]
}

// Only show significant events, not regular output
const TIMELINE_EVENT_TYPES = [
  'state_changed',
  'approval_request',
  'approval_response',
  'guidance_injected',
  'policy_violation',
]

interface TimelineConfig {
  icon: string
  bg: string
  ring: string
  label: string
}

function getEventConfig(event: SessionOutputEvent): TimelineConfig {
  switch (event.event_type) {
    case 'state_changed':
      return {
        icon: 'arrow',
        bg: 'bg-primary-500',
        ring: 'ring-primary-100',
        label: 'State Changed',
      }
    case 'approval_request':
      return {
        icon: 'question',
        bg: 'bg-amber-500',
        ring: 'ring-amber-100',
        label: 'Approval Requested',
      }
    case 'approval_response':
      return {
        icon: 'check',
        bg: 'bg-green-500',
        ring: 'ring-green-100',
        label: 'Approval Response',
      }
    case 'guidance_injected':
      return {
        icon: 'chat',
        bg: 'bg-blue-500',
        ring: 'ring-blue-100',
        label: 'Guidance Injected',
      }
    case 'policy_violation':
      return {
        icon: 'shield',
        bg: 'bg-red-500',
        ring: 'ring-red-100',
        label: 'Policy Violation',
      }
    default:
      return {
        icon: 'dot',
        bg: 'bg-secondary-400',
        ring: 'ring-secondary-100',
        label: event.event_type,
      }
  }
}

function TimelineIcon({ icon, bg, ring }: { icon: string; bg: string; ring: string }) {
  const base = `flex h-8 w-8 items-center justify-center rounded-full ring-4 ${bg} ${ring}`
  switch (icon) {
    case 'arrow':
      return (
        <span className={base}>
          <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6" />
          </svg>
        </span>
      )
    case 'question':
      return (
        <span className={base}>
          <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01" />
          </svg>
        </span>
      )
    case 'check':
      return (
        <span className={base}>
          <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
          </svg>
        </span>
      )
    case 'chat':
      return (
        <span className={base}>
          <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
        </span>
      )
    case 'shield':
      return (
        <span className={base}>
          <svg className="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </span>
      )
    default:
      return (
        <span className={base}>
          <span className="h-2 w-2 rounded-full bg-white" />
        </span>
      )
  }
}

function formatTime(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return ''
  }
}

function formatDate(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleDateString([], { month: 'short', day: 'numeric' })
  } catch {
    return ''
  }
}

export function SessionTimeline({ events }: SessionTimelineProps) {
  const timelineEvents = events.filter((e) => TIMELINE_EVENT_TYPES.includes(e.event_type))

  if (timelineEvents.length === 0) {
    return (
      <div className="rounded-lg border border-secondary-200 bg-white p-8 text-center">
        <p className="text-sm text-secondary-500">No significant events yet.</p>
      </div>
    )
  }

  return (
    <div className="rounded-lg border border-secondary-200 bg-white p-6 shadow-sm">
      <div className="flow-root">
        <ul className="-mb-8">
          {timelineEvents.map((event, idx) => {
            const config = getEventConfig(event)
            const isLast = idx === timelineEvents.length - 1
            return (
              <li key={event.id}>
                <div className="relative pb-8">
                  {/* Connector line */}
                  {!isLast && (
                    <span
                      className="absolute left-4 top-8 -ml-px h-full w-0.5 bg-secondary-200"
                      aria-hidden="true"
                    />
                  )}
                  <div className="relative flex items-start gap-4">
                    <TimelineIcon icon={config.icon} bg={config.bg} ring={config.ring} />
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center justify-between gap-4">
                        <p className="text-sm font-medium text-secondary-900">{config.label}</p>
                        <span className="shrink-0 text-xs text-secondary-500">
                          {formatDate(event.timestamp)} {formatTime(event.timestamp)}
                        </span>
                      </div>
                      <p className="mt-1 text-sm text-secondary-600">{event.content}</p>
                    </div>
                  </div>
                </div>
              </li>
            )
          })}
        </ul>
      </div>
    </div>
  )
}
