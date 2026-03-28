import { useEffect, useRef, useState, useCallback } from 'react'
import { getMachineLogs } from '../api/machineLogs'
import type { MachineLogEntry } from '../api/machineLogs'
import { useMachineLogStream } from '../hooks/useMachineLogStream'

interface MachineLogViewerProps {
  machineId: string
}

type ViewMode = 'live' | 'history'
type LevelFilter = 'all' | 'debug' | 'info' | 'warn' | 'error'

const LEVEL_OPTIONS: { value: LevelFilter; label: string }[] = [
  { value: 'all', label: 'All Levels' },
  { value: 'debug', label: 'Debug' },
  { value: 'info', label: 'Info' },
  { value: 'warn', label: 'Warn' },
  { value: 'error', label: 'Error' },
]

const PAGE_SIZE = 50

function levelClasses(level: string): string {
  switch (level.toLowerCase()) {
    case 'debug':
    case 'trace':
      return 'text-secondary-500'
    case 'info':
      return 'text-blue-400'
    case 'warn':
    case 'warning':
      return 'text-amber-400'
    case 'error':
    case 'fatal':
      return 'text-red-400'
    default:
      return 'text-secondary-400'
  }
}

function levelBadgeClasses(level: string): string {
  switch (level.toLowerCase()) {
    case 'debug':
    case 'trace':
      return 'bg-secondary-700 text-secondary-300'
    case 'info':
      return 'bg-blue-900/50 text-blue-300'
    case 'warn':
    case 'warning':
      return 'bg-amber-900/50 text-amber-300'
    case 'error':
    case 'fatal':
      return 'bg-red-900/50 text-red-300'
    default:
      return 'bg-secondary-700 text-secondary-300'
  }
}

function formatTimestamp(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return ''
  }
}

function matchesLevel(entry: MachineLogEntry, filter: LevelFilter): boolean {
  if (filter === 'all') return true
  const lvl = entry.level.toLowerCase()
  if (filter === 'warn') return lvl === 'warn' || lvl === 'warning'
  if (filter === 'error') return lvl === 'error' || lvl === 'fatal'
  return lvl === filter
}

export function MachineLogViewer({ machineId }: MachineLogViewerProps) {
  const [mode, setMode] = useState<ViewMode>('live')
  const [levelFilter, setLevelFilter] = useState<LevelFilter>('all')
  const [scrollLocked, setScrollLocked] = useState(false)

  // Live mode state
  const { logs: streamLogs, isConnected, error: streamError } = useMachineLogStream(machineId)

  // History mode state
  const [historyLogs, setHistoryLogs] = useState<MachineLogEntry[]>([])
  const [historyLoading, setHistoryLoading] = useState(false)
  const [historyError, setHistoryError] = useState<string | null>(null)
  const [historyOffset, setHistoryOffset] = useState(0)
  const [hasMore, setHasMore] = useState(true)

  const scrollRef = useRef<HTMLDivElement>(null)

  // Auto-scroll in live mode
  useEffect(() => {
    if (mode === 'live' && !scrollLocked && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [streamLogs, scrollLocked, mode])

  // Fetch history logs
  const fetchHistory = useCallback(async (offset: number, append: boolean) => {
    setHistoryLoading(true)
    setHistoryError(null)
    try {
      const levelParam = levelFilter !== 'all' ? levelFilter : undefined
      const data = await getMachineLogs(machineId, {
        level: levelParam,
        limit: PAGE_SIZE,
        offset,
      })
      if (append) {
        setHistoryLogs((prev) => [...prev, ...data])
      } else {
        setHistoryLogs(data)
      }
      setHasMore(data.length === PAGE_SIZE)
      setHistoryOffset(offset + data.length)
    } catch (err) {
      setHistoryError(err instanceof Error ? err.message : 'Failed to load logs')
    } finally {
      setHistoryLoading(false)
    }
  }, [machineId, levelFilter])

  // Load history when switching to history mode or changing filter
  useEffect(() => {
    if (mode === 'history') {
      setHistoryOffset(0)
      setHasMore(true)
      void fetchHistory(0, false)
    }
  }, [mode, fetchHistory])

  const handleLoadMore = () => {
    void fetchHistory(historyOffset, true)
  }

  const displayLogs = mode === 'live' ? streamLogs : historyLogs
  const filteredLogs = displayLogs.filter((entry) => matchesLevel(entry, levelFilter))

  return (
    <div className="flex flex-col rounded-lg border border-secondary-200 bg-secondary-950 shadow-sm overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between border-b border-secondary-700 bg-secondary-900 px-4 py-2">
        <div className="flex items-center gap-3">
          {/* Mode toggle */}
          <div className="flex rounded-md bg-secondary-800">
            <button
              type="button"
              onClick={() => setMode('live')}
              className={`rounded-l-md px-3 py-1 text-xs font-medium transition-colors ${
                mode === 'live'
                  ? 'bg-primary-600 text-white'
                  : 'text-secondary-300 hover:bg-secondary-700'
              }`}
            >
              Live
            </button>
            <button
              type="button"
              onClick={() => setMode('history')}
              className={`rounded-r-md px-3 py-1 text-xs font-medium transition-colors ${
                mode === 'history'
                  ? 'bg-primary-600 text-white'
                  : 'text-secondary-300 hover:bg-secondary-700'
              }`}
            >
              History
            </button>
          </div>

          {/* Connection indicator (live mode only) */}
          {mode === 'live' && (
            <span className="flex items-center gap-1.5 text-xs text-secondary-400">
              <span
                className={`inline-block h-2 w-2 rounded-full ${
                  isConnected ? 'bg-green-500' : 'bg-red-500'
                }`}
              />
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
          )}

          {/* Entry count */}
          <span className="text-xs text-secondary-500">
            {filteredLogs.length} {mode === 'live' ? 'entries' : 'loaded'}
          </span>
        </div>

        <div className="flex items-center gap-3">
          {/* Level filter */}
          <select
            value={levelFilter}
            onChange={(e) => setLevelFilter(e.target.value as LevelFilter)}
            className="rounded-md border-0 bg-secondary-800 px-2 py-1 text-xs text-secondary-300 focus:ring-1 focus:ring-primary-500"
          >
            {LEVEL_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>

          {/* Auto-scroll toggle (live mode only) */}
          {mode === 'live' && (
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
          )}
        </div>
      </div>

      {/* Error banner */}
      {(mode === 'live' ? streamError : historyError) && (
        <div className="border-b border-red-800 bg-red-900/40 px-4 py-1.5 text-xs text-red-300">
          {mode === 'live' ? streamError : historyError}
        </div>
      )}

      {/* Log area */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-4 font-mono text-sm leading-relaxed"
        style={{ maxHeight: '60vh', minHeight: '300px' }}
      >
        {filteredLogs.length === 0 ? (
          <p className="text-secondary-600 text-center py-8">
            {mode === 'live'
              ? isConnected
                ? 'Waiting for log entries...'
                : 'Not connected'
              : historyLoading
                ? 'Loading...'
                : 'No log entries found'}
          </p>
        ) : (
          <>
            {filteredLogs.map((entry) => (
              <div
                key={entry.id}
                className="flex gap-3 py-0.5 hover:bg-secondary-900/50"
              >
                <span className="shrink-0 text-secondary-600 select-none">
                  {formatTimestamp(entry.timestamp)}
                </span>
                <span
                  className={`shrink-0 inline-flex items-center rounded px-1.5 py-0 text-xs font-medium uppercase ${levelBadgeClasses(entry.level)}`}
                >
                  {entry.level.toUpperCase().padEnd(5)}
                </span>
                <span className="shrink-0 text-secondary-500 text-xs leading-relaxed">
                  {entry.target}
                </span>
                <span className={levelClasses(entry.level)}>
                  {entry.message}
                </span>
              </div>
            ))}

            {/* Load More button (history mode only) */}
            {mode === 'history' && hasMore && (
              <div className="flex justify-center pt-4">
                <button
                  type="button"
                  onClick={handleLoadMore}
                  disabled={historyLoading}
                  className="rounded-md bg-secondary-800 px-4 py-2 text-xs font-medium text-secondary-300 hover:bg-secondary-700 disabled:opacity-50"
                >
                  {historyLoading ? 'Loading...' : 'Load More'}
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}
