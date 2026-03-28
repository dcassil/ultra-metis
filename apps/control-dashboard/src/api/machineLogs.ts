import { apiClient } from './client'

export interface MachineLogEntry {
  id: string
  machine_id: string
  timestamp: string
  level: string
  target: string
  message: string
  fields_json: string | null
}

export async function getMachineLogs(
  machineId: string,
  params?: {
    level?: string
    since?: string
    target?: string
    limit?: number
    offset?: number
  },
): Promise<MachineLogEntry[]> {
  const queryParams: Record<string, string | number> = {}
  if (params?.level) queryParams.level = params.level
  if (params?.since) queryParams.since = params.since
  if (params?.target) queryParams.target = params.target
  if (params?.limit !== undefined) queryParams.limit = params.limit
  if (params?.offset !== undefined) queryParams.offset = params.offset

  const response = await apiClient.get<MachineLogEntry[]>(
    `/api/machines/${machineId}/logs`,
    { params: queryParams },
  )
  return response.data
}
