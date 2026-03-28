import { apiClient } from './client'

// --- Types ---

export interface MachinePolicy {
  id: string
  machine_id: string
  allowed_categories: string[]
  blocked_categories: string[]
  max_autonomy_level: string
  session_mode: string
  require_approval_for: string[]
  created_at: string
  updated_at: string
}

export interface UpdatePolicyRequest {
  allowed_categories?: string[]
  blocked_categories?: string[]
  max_autonomy_level?: string
  session_mode?: string
  require_approval_for?: string[]
}

/** @deprecated Use MachinePolicy instead */
export type Policy = MachinePolicy

// --- Constants ---

export const ACTION_CATEGORIES = [
  { value: 'read_files', label: 'Read Files' },
  { value: 'write_files', label: 'Write Files' },
  { value: 'run_tests', label: 'Run Tests' },
  { value: 'run_builds', label: 'Run Builds' },
  { value: 'git_operations', label: 'Git Operations' },
  { value: 'install_packages', label: 'Install Packages' },
  { value: 'network_access', label: 'Network Access' },
  { value: 'worktree_operations', label: 'Worktree Operations' },
  { value: 'shell_execution', label: 'Shell Execution' },
] as const

export const AUTONOMY_LEVELS = [
  { value: 'normal', label: 'Normal' },
  { value: 'stricter', label: 'Stricter' },
  { value: 'autonomous', label: 'Autonomous' },
] as const

export const SESSION_MODES = [
  { value: 'normal', label: 'Normal' },
  { value: 'restricted', label: 'Restricted' },
  { value: 'elevated', label: 'Elevated' },
] as const

// --- API Functions ---

export async function getMachinePolicy(machineId: string): Promise<MachinePolicy> {
  const response = await apiClient.get<MachinePolicy>(`/api/machines/${machineId}/policy`)
  return response.data
}

export async function updateMachinePolicy(
  machineId: string,
  data: UpdatePolicyRequest,
): Promise<MachinePolicy> {
  const response = await apiClient.put<MachinePolicy>(`/api/machines/${machineId}/policy`, data)
  return response.data
}

export async function getRepoPolicy(machineId: string, repoPath: string): Promise<MachinePolicy> {
  const response = await apiClient.get<MachinePolicy>(`/api/machines/${machineId}/repo-policy`, {
    params: { repo_path: repoPath },
  })
  return response.data
}

export async function updateRepoPolicy(
  machineId: string,
  repoPath: string,
  data: UpdatePolicyRequest,
): Promise<MachinePolicy> {
  const response = await apiClient.put<MachinePolicy>(`/api/machines/${machineId}/repo-policy`, data, {
    params: { repo_path: repoPath },
  })
  return response.data
}

export async function getEffectivePolicy(
  machineId: string,
  repoPath: string,
): Promise<MachinePolicy> {
  const response = await apiClient.get<MachinePolicy>(`/api/machines/${machineId}/policy/effective`, {
    params: { repo_path: repoPath },
  })
  return response.data
}

// --- Violation Types ---

export interface PolicyViolationRecord {
  id: string
  session_id: string | null
  machine_id: string
  user_id: string
  action: string
  policy_scope: string
  reason: string
  repo_path: string | null
  timestamp: string
}

export interface ViolationsListResponse {
  violations: PolicyViolationRecord[]
  total: number
}

export interface ListViolationsParams {
  machine_id?: string
  session_id?: string
  limit?: number
  offset?: number
}

// --- Violation API Functions ---

export async function listViolations(params?: ListViolationsParams): Promise<ViolationsListResponse> {
  const response = await apiClient.get<ViolationsListResponse>('/api/policy-violations', { params })
  return response.data
}

export async function listSessionViolations(sessionId: string): Promise<PolicyViolationRecord[]> {
  const response = await apiClient.get<PolicyViolationRecord[]>(`/api/sessions/${sessionId}/violations`)
  return response.data
}
