import { apiClient } from './client'

// --- Request / Response Types ---

export interface CreateSessionRequest {
  machine_id: string
  repo_path: string
  title: string
  instructions: string
  autonomy_level?: 'normal' | 'stricter' | 'autonomous'
  work_item_id?: string
  context?: string
}

export interface CreateSessionResponse {
  id: string
  state: string
}

export interface SessionResponse {
  id: string
  machine_id: string
  repo_path: string
  title: string
  instructions: string
  autonomy_level: string
  work_item_id?: string
  context?: string
  state: string
  outcome_status?: string | null
  created_at: string
  updated_at: string
  started_at?: string
  completed_at?: string
}

export interface SessionListResponse {
  sessions: SessionResponse[]
  total: number
}

export interface ListSessionsParams {
  machine_id?: string
  repo_path?: string
  state?: string
  limit?: number
  offset?: number
  outcome?: string
  search?: string
  from_date?: string
  to_date?: string
  sort_by?: string
  sort_order?: string
}

// --- API Functions ---

export async function createSession(data: CreateSessionRequest): Promise<CreateSessionResponse> {
  const response = await apiClient.post<CreateSessionResponse>('/api/sessions', data)
  return response.data
}

export async function getSession(id: string): Promise<SessionResponse> {
  const response = await apiClient.get<SessionResponse>(`/api/sessions/${id}`)
  return response.data
}

export async function listSessions(params?: ListSessionsParams): Promise<SessionListResponse> {
  const response = await apiClient.get<SessionListResponse>('/api/sessions', { params })
  return response.data
}

// --- Control Actions ---

export async function stopSession(id: string): Promise<void> {
  await apiClient.post(`/api/sessions/${id}/stop`)
}

export async function forceStopSession(id: string): Promise<void> {
  await apiClient.post(`/api/sessions/${id}/force-stop`)
}

export async function pauseSession(id: string): Promise<void> {
  await apiClient.post(`/api/sessions/${id}/pause`)
}

export async function resumeSession(id: string): Promise<void> {
  await apiClient.post(`/api/sessions/${id}/resume`)
}
