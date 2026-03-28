import { apiClient } from './client'
import type { SessionResponse } from './sessions'

// --- Types ---

export interface SessionOutcome {
  id: string
  session_id: string
  status: string // success, failure, partial
  summary: string
  artifacts: unknown
  next_steps: unknown
  event_count: number
  intervention_count: number
  duration_seconds: number
  created_at: string
}

// --- API Functions ---

export async function getSessionOutcome(sessionId: string): Promise<SessionOutcome> {
  const response = await apiClient.get<SessionOutcome>(
    `/api/sessions/${sessionId}/outcome`,
  )
  return response.data
}

export async function getHistorySession(sessionId: string): Promise<SessionResponse> {
  const response = await apiClient.get<SessionResponse>(
    `/api/sessions/${sessionId}`,
  )
  return response.data
}
