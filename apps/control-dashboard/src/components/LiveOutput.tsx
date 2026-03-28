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

interface ClaudeResult {
  type: 'result'
  subtype: string
  is_error: boolean
  duration_ms: number
  total_cost_usd: number
  result: string
  num_turns: number
}

/** Try to parse a content string as a Claude CLI JSON result. */
function parseClaudeResult(content: string): ClaudeResult | null {
  const trimmed = content.trim()
  if (!trimmed.startsWith('{')) return null
  try {
    const parsed = JSON.parse(trimmed) as Record<string, unknown>
    if (parsed.type === 'result' && typeof parsed.result === 'string') {
      return parsed as unknown as ClaudeResult
    }
  } catch {
    // not JSON
  }
  return null
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
      {/* Toolbar */}
      <div className="flex items-center justify-between border-b border-secondary-700 bg-secondary-900 px-4 py-2">
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
        className="flex-1 overflow-y-auto p-4 font-mono text-sm leading-relaxed"
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
            const claudeResult = parseClaudeResult(evt.content)

            if (claudeResult) {
              const durationSec = (claudeResult.duration_ms / 1000).toFixed(1)
              const cost = claudeResult.total_cost_usd.toFixed(4)
              return (
                <div key={evt.id} className="py-2">
                  <div className="flex items-center gap-3 mb-2">
                    <span className="shrink-0 text-secondary-600 select-none">
                      {formatTimestamp(evt.timestamp)}
                    </span>
                    <span className={`inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-medium ${
                      claudeResult.is_error
                        ? 'bg-red-900/50 text-red-300'
                        : 'bg-green-900/50 text-green-300'
                    }`}>
                      {claudeResult.is_error ? 'Error' : 'Complete'}
                    </span>
                    <span className="text-xs text-secondary-500">
                      {durationSec}s &middot; {claudeResult.num_turns} turns &middot; ${cost}
                    </span>
                  </div>
                  <div className="ml-[4.5rem] rounded-md border border-secondary-700 bg-secondary-900 p-3">
                    <pre className="whitespace-pre-wrap text-secondary-200 text-sm leading-relaxed">
                      {claudeResult.result}
                    </pre>
                  </div>
                </div>
              )
            }

            return (
              <div key={evt.id} className={`flex gap-3 py-0.5 hover:bg-secondary-900/50 ${isGuidance ? 'bg-indigo-950/30' : ''}`}>
                <span className="shrink-0 text-secondary-600 select-none">
                  {formatTimestamp(evt.timestamp)}
                </span>
                {isGuidance ? (
                  <span className="italic text-indigo-400">
                    <span className="font-semibold not-italic text-indigo-300">You: </span>
                    {evt.content}
                  </span>
                ) : (
                  <span className={categoryClasses(evt.category)}>{evt.content}</span>
                )}
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
