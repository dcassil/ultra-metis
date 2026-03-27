import { apiClient } from './client'

export interface HealthStatus {
  status: 'ok' | 'degraded' | 'error'
  version?: string
  uptime?: number
}

export async function checkHealth(): Promise<HealthStatus> {
  try {
    const response = await apiClient.get<HealthStatus>('/health')
    return response.data
  } catch {
    return { status: 'error' }
  }
}
