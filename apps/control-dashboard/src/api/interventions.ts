import { apiClient } from './client'

// --- Types ---

export interface PendingApproval {
  id: string
  session_id: string
  question: string
  options: string // JSON array string e.g. '["Allow","Deny"]'
  context: string | null
  status: string
  response_choice: string | null
  response_note: string | null
  created_at: string
  responded_at: string | null
}

export interface RespondToApprovalRequest {
  approval_id: string
  choice: string
  note?: string
}

export interface InjectGuidanceRequest {
  message: string
  injection_type: 'normal' | 'side_note' | 'interrupt'
}

// --- API Functions ---

export async function listPendingApprovals(sessionId: string): Promise<PendingApproval[]> {
  const response = await apiClient.get<PendingApproval[]>(
    `/api/sessions/${sessionId}/approvals`,
  )
  return response.data
}

export async function respondToApproval(
  sessionId: string,
  approvalId: string,
  choice: string,
  note?: string,
): Promise<void> {
  await apiClient.post(`/api/sessions/${sessionId}/respond`, {
    approval_id: approvalId,
    choice,
    note,
  })
}

export async function injectGuidance(
  sessionId: string,
  message: string,
  injectionType: string,
): Promise<void> {
  await apiClient.post(`/api/sessions/${sessionId}/inject`, {
    message,
    injection_type: injectionType,
  })
}
