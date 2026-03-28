import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import type { Notification } from '../api/notifications'
import { listNotifications, markRead, dismissNotification } from '../api/notifications'
import { Button } from '../components/ui/Button'
import { RelativeTime } from '../components/RelativeTime'

function PriorityDot({ priority }: { priority: string }) {
  if (priority === 'urgent') {
    return (
      <span className="relative mr-3 flex h-3 w-3 shrink-0">
        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-danger-400 opacity-75" />
        <span className="relative inline-flex h-3 w-3 rounded-full bg-danger-500" />
      </span>
    )
  }
  if (priority === 'high') {
    return <span className="mr-3 h-3 w-3 shrink-0 rounded-full bg-warning-500" />
  }
  return <span className="mr-3 h-3 w-3 shrink-0 rounded-full bg-primary-500" />
}

export default function NotificationsPage() {
  const navigate = useNavigate()
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchNotifications = useCallback(async () => {
    try {
      setLoading(true)
      const data = await listNotifications()
      // Sort unread first, then by created_at descending
      data.sort((a, b) => {
        const aRead = a.read_at ? 1 : 0
        const bRead = b.read_at ? 1 : 0
        if (aRead !== bRead) return aRead - bRead
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
      })
      setNotifications(data)
      setError(null)
    } catch {
      setError('Failed to load notifications')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void fetchNotifications()
  }, [fetchNotifications])

  const handleMarkRead = useCallback(
    async (e: React.MouseEvent, id: string) => {
      e.stopPropagation()
      try {
        await markRead(id)
        setNotifications((prev) =>
          prev.map((n) => (n.id === id ? { ...n, read_at: new Date().toISOString() } : n)),
        )
      } catch {
        // Silently ignore — user can retry
      }
    },
    [],
  )

  const handleDismiss = useCallback(
    async (e: React.MouseEvent, id: string) => {
      e.stopPropagation()
      try {
        await dismissNotification(id)
        setNotifications((prev) => prev.filter((n) => n.id !== id))
      } catch {
        // Silently ignore — user can retry
      }
    },
    [],
  )

  const handleClick = useCallback(
    (notification: Notification) => {
      if (notification.deep_link) {
        navigate(notification.deep_link)
      }
    },
    [navigate],
  )

  if (error && !loading && notifications.length === 0) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold text-secondary-900">Notifications</h2>
        <div className="flex items-center justify-center py-24">
          <div className="rounded-lg border border-danger-200 bg-danger-50 px-8 py-12 text-center">
            <p className="text-sm text-danger-700">{error}</p>
            <button
              type="button"
              onClick={() => void fetchNotifications()}
              className="mt-3 text-sm font-medium text-primary-600 hover:text-primary-700"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-semibold text-secondary-900">Notifications</h2>
      </div>

      <div className="rounded-lg border border-secondary-200 bg-white shadow-sm">
        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="text-sm text-secondary-500">Loading notifications...</div>
          </div>
        ) : notifications.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <p className="text-sm text-secondary-500">No notifications.</p>
          </div>
        ) : (
          <ul className="divide-y divide-secondary-100">
            {notifications.map((notification) => {
              const isUnread = !notification.read_at
              return (
                <li
                  key={notification.id}
                  onClick={() => handleClick(notification)}
                  className={[
                    'flex items-start gap-x-4 px-4 py-4 sm:px-6 transition-colors',
                    notification.deep_link ? 'cursor-pointer hover:bg-secondary-50' : '',
                    isUnread ? 'bg-primary-50/40' : '',
                  ].join(' ')}
                >
                  <PriorityDot priority={notification.priority} />

                  <div className="min-w-0 flex-1">
                    <p
                      className={[
                        'text-sm',
                        isUnread
                          ? 'font-semibold text-secondary-900'
                          : 'font-medium text-secondary-700',
                      ].join(' ')}
                    >
                      {notification.title}
                    </p>
                    <p className="mt-0.5 line-clamp-2 text-sm text-secondary-500">
                      {notification.body}
                    </p>
                    <p className="mt-1 text-xs text-secondary-400">
                      <RelativeTime timestamp={notification.created_at} />
                    </p>
                  </div>

                  <div className="flex shrink-0 items-center gap-x-2">
                    {isUnread && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => void handleMarkRead(e, notification.id)}
                      >
                        Mark Read
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => void handleDismiss(e, notification.id)}
                    >
                      Dismiss
                    </Button>
                  </div>
                </li>
              )
            })}
          </ul>
        )}
      </div>
    </div>
  )
}
