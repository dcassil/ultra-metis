// Re-export the shared API client from the api module
// Auth configuration (token injection, 401 handling) lives in api/client.ts
export { apiClient } from '../api/client'
