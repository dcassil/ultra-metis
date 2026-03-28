import { apiClient } from './client'

// --- Types ---

export interface SessionOutputEvent {
  id: string
  session_id: string
  event_type: string // output_line, approval_request, approval_response, guidance_injected, state_changed, policy_violation
  category: string | null // info, warning, error, summary
  content: string
  metadata: unknown
  sequence_num: number
  timestamp: string
}

export interface EventsResponse {
  events: SessionOutputEvent[]
}

// --- API Functions ---

export async function getSessionEvents(
  sessionId: string,
  sinceSequence?: number,
  limit?: number,
): Promise<EventsResponse> {
  const params: Record<string, string | number> = {}
  if (sinceSequence !== undefined) params.since_sequence = sinceSequence
  if (limit !== undefined) params.limit = limit
  const response = await apiClient.get<EventsResponse>(
    `/api/sessions/${sessionId}/events`,
    { params },
  )
  return response.data
}
