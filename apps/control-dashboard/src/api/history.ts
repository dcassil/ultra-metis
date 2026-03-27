// Stub API module for Session History — SMET-I-0043

export interface HistoryEntry {
  id: string
  sessionId: string
  machineId: string
  action: string
  timestamp: string
}

export interface HistoryFilters {
  machineId?: string
  sessionId?: string
  from?: string
  to?: string
}

export function listHistoryEntries(_filters: HistoryFilters): Promise<HistoryEntry[]> {
  throw new Error('Not implemented — see SMET-I-0043')
}

export function getHistoryEntry(_id: string): Promise<HistoryEntry> {
  throw new Error('Not implemented — see SMET-I-0043')
}
