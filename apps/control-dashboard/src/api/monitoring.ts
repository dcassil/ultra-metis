// Stub API module for Monitoring — SMET-I-0041

export interface Metrics {
  machineId: string
  cpu: number
  memory: number
  activeSessions: number
}

export interface Alert {
  id: string
  severity: 'info' | 'warning' | 'critical'
  message: string
  timestamp: string
}

export function getMetrics(_machineId: string): Promise<Metrics> {
  throw new Error('Not implemented — see SMET-I-0041')
}

export function getAlerts(): Promise<Alert[]> {
  throw new Error('Not implemented — see SMET-I-0041')
}
