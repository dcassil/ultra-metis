// Stub API module for Policy management — SMET-I-0044

export interface Policy {
  id: string
  name: string
  description: string
  enabled: boolean
  rules: Record<string, unknown>
}

export function listPolicies(): Promise<Policy[]> {
  throw new Error('Not implemented — see SMET-I-0044')
}

export function getPolicy(_id: string): Promise<Policy> {
  throw new Error('Not implemented — see SMET-I-0044')
}

export function createPolicy(_data: Omit<Policy, 'id'>): Promise<Policy> {
  throw new Error('Not implemented — see SMET-I-0044')
}

export function updatePolicy(_id: string, _data: Partial<Policy>): Promise<Policy> {
  throw new Error('Not implemented — see SMET-I-0044')
}
