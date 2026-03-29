import { useEffect, useRef, useState } from 'react'
import type { SessionOutputEvent } from '../api/events'

interface LiveOutputProps {
  events: SessionOutputEvent[]
  isConnected: boolean
  error: string | null
}

function categoryClasses(category: string | null): string {
  switch (category) {
    case 'warning':
      return 'text-amber-600'
    case 'error':
      return 'text-red-500'
    case 'summary':
      return 'text-blue-600 font-semibold'
    case 'info':
    default:
      return 'text-secondary-400'
  }
}

// -- Claude CLI JSON event types --

interface ClaudeContentBlock {
  type: string
  text?: string
  id?: string
  name?: string
  input?: Record<string, unknown>
}

interface ClaudeAssistantEvent {
  type: 'assistant'
  message: {
    content: ClaudeContentBlock[]
    [key: string]: unknown
  }
}

interface ClaudeToolUseEvent {
  type: 'tool_use'
  name: string
  input: Record<string, unknown>
  [key: string]: unknown
}

interface ClaudeToolResultEvent {
  type: 'tool_result'
  tool_use_id?: string
  content?: string
  is_error?: boolean
  [key: string]: unknown
}

interface ClaudeSystemEvent {
  type: 'system'
  message?: string
  [key: string]: unknown
}

interface ClaudeResult {
  type: 'result'
  subtype: string
  is_error: boolean
  duration_ms: number
  total_cost_usd: number
  result: string
  num_turns: number
}

type ClaudeEvent =
  | ClaudeAssistantEvent
  | ClaudeToolUseEvent
  | ClaudeToolResultEvent
  | ClaudeSystemEvent
  | ClaudeResult

/** Try to parse content as a Claude CLI JSON event. */
function parseClaudeEvent(content: string): ClaudeEvent | null {
  const trimmed = content.trim()
  if (!trimmed.startsWith('{')) return null
  try {
    const parsed = JSON.parse(trimmed) as Record<string, unknown>
    if (typeof parsed.type === 'string') {
      return parsed as unknown as ClaudeEvent
    }
  } catch {
    // not JSON
  }
  return null
}

/** Extract readable text from an assistant message's content blocks. */
function extractAssistantText(event: ClaudeAssistantEvent): string {
  if (!event.message?.content) return ''
  return event.message.content
    .filter((block) => block.type === 'text' && block.text)
    .map((block) => block.text!)
    .join('\n')
}

/** Build a short summary for a tool use event. */
function toolUseSummary(event: ClaudeToolUseEvent): string {
  const input = event.input
  if (!input) return ''
  // Common tool patterns
  if (typeof input.command === 'string') return truncate(input.command as string, 120)
  if (typeof input.file_path === 'string') return input.file_path as string
  if (typeof input.pattern === 'string') return truncate(input.pattern as string, 80)
  if (typeof input.query === 'string') return truncate(input.query as string, 80)
  if (typeof input.prompt === 'string') return truncate(input.prompt as string, 80)
  // Fallback: first string value
  for (const val of Object.values(input)) {
    if (typeof val === 'string' && val.length > 0) return truncate(val, 80)
  }
  return ''
}

function truncate(s: string, max: number): string {
  if (s.length <= max) return s
  return s.slice(0, max) + '\u2026'
}

/** Lines to suppress from the terminal output. */
function isSuppressedLine(content: string): boolean {
  return content.includes('no stdin data received') && content.includes('proceeding without it')
}

function formatTimestamp(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return ''
  }
}

/** Render a parsed Claude CLI event into styled JSX. */
function renderClaudeEvent(event: ClaudeEvent, timestamp: string, eventId: string): React.ReactElement | null {
  switch (event.type) {
    case 'result': {
      const r = event as ClaudeResult
      if (typeof r.result !== 'string') return null
      const durationSec = (r.duration_ms / 1000).toFixed(1)
      const cost = r.total_cost_usd.toFixed(4)
      return (
        <div key={eventId} className="py-2">
          <div className="flex items-center gap-3 mb-2">
            <span className="shrink-0 text-secondary-600 select-none">
              {formatTimestamp(timestamp)}
            </span>
            <span className={`inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-medium ${
              r.is_error
                ? 'bg-red-900/50 text-red-300'
                : 'bg-green-900/50 text-green-300'
            }`}>
              {r.is_error ? 'Error' : 'Complete'}
            </span>
            <span className="text-xs text-secondary-500">
              {durationSec}s &middot; {r.num_turns} turns &middot; ${cost}
            </span>
          </div>
          <div className="ml-[4.5rem] rounded-md border border-secondary-700 bg-secondary-900 p-3">
            <pre className="whitespace-pre-wrap text-secondary-200 text-sm leading-relaxed">
              {r.result}
            </pre>
          </div>
        </div>
      )
    }

    case 'assistant': {
      const text = extractAssistantText(event as ClaudeAssistantEvent)
      if (!text) return null
      return (
        <div key={eventId} className="py-1.5">
          <div className="flex gap-3">
            <span className="shrink-0 text-secondary-600 select-none">
              {formatTimestamp(timestamp)}
            </span>
            <div className="min-w-0 flex-1 border-l-2 border-emerald-700/60 pl-3">
              <pre className="whitespace-pre-wrap text-emerald-300 text-sm leading-relaxed">
                {text}
              </pre>
            </div>
          </div>
        </div>
      )
    }

    case 'tool_use': {
      const e = event as ClaudeToolUseEvent
      const summary = toolUseSummary(e)
      return (
        <div key={eventId} className="flex gap-3 py-0.5 hover:bg-secondary-900/50">
          <span className="shrink-0 text-secondary-600 select-none">
            {formatTimestamp(timestamp)}
          </span>
          <span className="text-secondary-500">
            <code className="rounded bg-secondary-800 px-1.5 py-0.5 text-xs font-mono text-amber-400">
              {e.name}
            </code>
            {summary && (
              <span className="ml-2 text-secondary-400 text-sm">{summary}</span>
            )}
          </span>
        </div>
      )
    }

    case 'tool_result': {
      const e = event as ClaudeToolResultEvent
      const content = typeof e.content === 'string' ? e.content : ''
      if (!content) return null
      return (
        <div key={eventId} className="py-0.5">
          <div className="flex gap-3">
            <span className="shrink-0 text-secondary-600 select-none">
              {formatTimestamp(timestamp)}
            </span>
            <div className="min-w-0 flex-1 ml-2">
              <pre className={`whitespace-pre-wrap text-xs leading-relaxed ${
                e.is_error ? 'text-red-400' : 'text-secondary-500'
              }`}>
                {truncate(content, 500)}
              </pre>
            </div>
          </div>
        </div>
      )
    }

    case 'system': {
      const e = event as ClaudeSystemEvent
      const msg = e.message || JSON.stringify(e)
      return (
        <div key={eventId} className="flex gap-3 py-0.5 hover:bg-secondary-900/50">
          <span className="shrink-0 text-secondary-600 select-none">
            {formatTimestamp(timestamp)}
          </span>
          <span className="italic text-secondary-500 text-sm">{msg}</span>
        </div>
      )
    }

    default:
      return null
  }
}

export function LiveOutput({ events, isConnected, error }: LiveOutputProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const [scrollLocked, setScrollLocked] = useState(false)

  // Auto-scroll to bottom when new events arrive (unless scroll-locked)
  useEffect(() => {
    if (!scrollLocked && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [events, scrollLocked])

  return (
    <div className="flex flex-col rounded-lg border border-secondary-200 bg-secondary-950 shadow-sm overflow-hidden">
      {/* Toolbar — sticky at top so controls stay visible while scrolling */}
      <div className="sticky top-0 z-10 flex items-center justify-between border-b border-secondary-700 bg-secondary-900 px-3 py-2 sm:px-4">
        <div className="flex items-center gap-3">
          {/* Connection status */}
          <span className="flex items-center gap-1.5 text-xs text-secondary-400">
            <span
              className={`inline-block h-2 w-2 rounded-full ${
                isConnected ? 'bg-green-500' : 'bg-red-500'
              }`}
            />
            {isConnected ? 'Connected' : 'Disconnected'}
          </span>
          {/* Event count */}
          <span className="text-xs text-secondary-500">{events.length} events</span>
        </div>
        <button
          type="button"
          onClick={() => setScrollLocked((v) => !v)}
          className={`rounded px-2 py-1 text-xs font-medium transition-colors ${
            scrollLocked
              ? 'bg-amber-600 text-white'
              : 'bg-secondary-700 text-secondary-300 hover:bg-secondary-600'
          }`}
        >
          {scrollLocked ? 'Scroll Locked' : 'Auto-scroll'}
        </button>
      </div>

      {/* Error banner */}
      {error && (
        <div className="border-b border-red-800 bg-red-900/40 px-4 py-1.5 text-xs text-red-300">
          {error}
        </div>
      )}

      {/* Log area */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-3 sm:p-4 font-mono text-xs sm:text-sm leading-relaxed break-words overflow-wrap-anywhere"
        style={{ maxHeight: '60vh', minHeight: '300px' }}
      >
        {events.length === 0 ? (
          <p className="text-secondary-600 text-center py-8">
            {isConnected ? 'Waiting for output...' : 'Not connected'}
          </p>
        ) : (
          events.map((evt) => {
            if (isSuppressedLine(evt.content)) return null

            const isGuidance = evt.event_type === 'guidance_injected'

            // Try to parse as a Claude CLI JSON event
            const claudeEvent = parseClaudeEvent(evt.content)
            if (claudeEvent) {
              const rendered = renderClaudeEvent(claudeEvent, evt.timestamp, evt.id)
              if (rendered) return rendered
              // If renderClaudeEvent returned null (unknown type), fall through to plain text
            }

            // Guidance injection events
            if (isGuidance) {
              return (
                <div key={evt.id} className="flex gap-3 py-0.5 hover:bg-secondary-900/50 bg-indigo-950/30">
                  <span className="shrink-0 text-secondary-600 select-none">
                    {formatTimestamp(evt.timestamp)}
                  </span>
                  <span className="italic text-indigo-400">
                    <span className="font-semibold not-italic text-indigo-300">You: </span>
                    {evt.content}
                  </span>
                </div>
              )
            }

            // Default: plain text rendering
            return (
              <div key={evt.id} className="flex gap-3 py-0.5 hover:bg-secondary-900/50">
                <span className="shrink-0 text-secondary-600 select-none">
                  {formatTimestamp(evt.timestamp)}
                </span>
                <span className={categoryClasses(evt.category)}>{evt.content}</span>
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
