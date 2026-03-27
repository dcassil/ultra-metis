// Stub API module for Session management — SMET-I-0040

export interface Session {
  id: string
  machineId: string
  status: 'running' | 'paused' | 'completed' | 'failed'
  startedAt: string
  stoppedAt?: string
}

export function listSessions(): Promise<Session[]> {
  throw new Error('Not implemented — see SMET-I-0040')
}

export function getSession(_id: string): Promise<Session> {
  throw new Error('Not implemented — see SMET-I-0040')
}

export function createSession(_data: { machineId: string; repo: string }): Promise<Session> {
  throw new Error('Not implemented — see SMET-I-0040')
}

export function stopSession(_id: string): Promise<void> {
  throw new Error('Not implemented — see SMET-I-0040')
}
