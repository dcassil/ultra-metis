import { apiClient } from './client'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface PlanningDocument {
  short_code: string
  title: string
  document_type: string
  phase: string
  parent_id: string | null
  archived: boolean
}

export interface DocumentDetail extends PlanningDocument {
  content: string
  children: PlanningDocument[]
}

export interface HierarchyNode {
  short_code: string
  title: string
  document_type: string
  phase: string
  children: HierarchyNode[]
}

export interface Rule {
  name: string
  scope: string
  description: string
  protection_level: string
}

export interface QualityRecord {
  short_code: string
  title: string
  gate_status: string
  details: string
}

// ---------------------------------------------------------------------------
// API functions
// ---------------------------------------------------------------------------

export interface ListDocumentsParams {
  document_type?: string
  phase?: string
  parent_id?: string
  include_archived?: boolean
}

export async function listDocuments(params?: ListDocumentsParams): Promise<PlanningDocument[]> {
  const response = await apiClient.get<PlanningDocument[]>('/api/planning/documents', { params })
  return response.data
}

export async function getDocument(shortCode: string): Promise<DocumentDetail> {
  const response = await apiClient.get<DocumentDetail>(`/api/planning/documents/${shortCode}`)
  return response.data
}

export async function searchDocuments(
  q: string,
  documentType?: string,
  limit?: number,
): Promise<PlanningDocument[]> {
  const response = await apiClient.get<PlanningDocument[]>('/api/planning/documents/search', {
    params: { q, document_type: documentType, limit },
  })
  return response.data
}

export async function getHierarchy(): Promise<HierarchyNode[]> {
  const response = await apiClient.get<HierarchyNode[]>('/api/planning/hierarchy')
  return response.data
}

export async function getRules(scope?: string): Promise<Rule[]> {
  const response = await apiClient.get<Rule[]>('/api/planning/rules', {
    params: scope ? { scope } : undefined,
  })
  return response.data
}

export async function getQualityRecords(shortCode: string): Promise<QualityRecord[]> {
  const response = await apiClient.get<QualityRecord[]>(`/api/planning/quality/${shortCode}`)
  return response.data
}
