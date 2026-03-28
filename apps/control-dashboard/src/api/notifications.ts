import { apiClient } from './client'

// --- Types ---

export interface Notification {
  id: string
  user_id: string
  session_id: string | null
  notification_type: string
  priority: string // 'urgent' | 'high' | 'normal'
  title: string
  body: string
  deep_link: string | null
  read_at: string | null
  dismissed_at: string | null
  created_at: string
}

interface UnreadCountResponse {
  count: number
}

// --- API Functions ---

export async function listNotifications(limit?: number, offset?: number): Promise<Notification[]> {
  const params: Record<string, number> = {}
  if (limit !== undefined) params.limit = limit
  if (offset !== undefined) params.offset = offset
  const response = await apiClient.get<Notification[]>('/api/notifications', { params })
  return response.data
}

export async function markRead(id: string): Promise<void> {
  await apiClient.post(`/api/notifications/${id}/read`)
}

export async function dismissNotification(id: string): Promise<void> {
  await apiClient.post(`/api/notifications/${id}/dismiss`)
}

export async function getUnreadCount(): Promise<number> {
  const response = await apiClient.get<UnreadCountResponse>('/api/notifications/unread-count')
  return response.data.count
}
