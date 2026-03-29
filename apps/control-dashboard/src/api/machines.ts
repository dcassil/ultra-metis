import { apiClient } from './client'

export interface Machine {
  id: string
  name: string
  platform: string
  status: 'pending' | 'trusted' | 'revoked'
  trust_tier: string
  connectivity_status: 'online' | 'stale' | 'offline' | 'unknown'
  repos_count: number
  last_heartbeat: string | null
  created_at: string
}

export interface MachineDetail extends Machine {
  metadata: Record<string, unknown>
  repos: Array<{ id: string; repo_name: string; repo_path: string; cadre_managed: boolean }>
  active_sessions: unknown[]
}

export async function listMachines(): Promise<Machine[]> {
  const response = await apiClient.get<Machine[]>('/api/machines')
  return response.data
}

export async function getMachine(id: string): Promise<MachineDetail> {
  const response = await apiClient.get<MachineDetail>(`/api/machines/${id}`)
  return response.data
}

export async function approveMachine(id: string): Promise<void> {
  await apiClient.post(`/api/machines/${id}/approve`)
}

export async function revokeMachine(id: string): Promise<void> {
  await apiClient.post(`/api/machines/${id}/revoke`)
}

export async function deleteMachine(id: string): Promise<void> {
  await apiClient.delete(`/api/machines/${id}`)
}

export async function deleteOfflineMachines(): Promise<{ deleted_count: number }> {
  const response = await apiClient.delete<{ deleted_count: number }>('/api/machines/offline')
  return response.data
}
