import { createContext, useContext } from 'react'
import type { ReactNode } from 'react'
import { useHealthCheck } from '../hooks/useHealthCheck'
import type { ConnectionStatus } from '../hooks/useHealthCheck'

interface HealthContextValue {
  status: ConnectionStatus
  lastChecked: Date | null
  error: string | null
}

const HealthContext = createContext<HealthContextValue | null>(null)

export function HealthProvider({ children }: { children: ReactNode }) {
  const health = useHealthCheck()
  return <HealthContext.Provider value={health}>{children}</HealthContext.Provider>
}

export function useHealth(): HealthContextValue {
  const ctx = useContext(HealthContext)
  if (!ctx) {
    throw new Error('useHealth must be used within a HealthProvider')
  }
  return ctx
}

export function useIsOnline(): boolean {
  const { status } = useHealth()
  return status === 'connected'
}
