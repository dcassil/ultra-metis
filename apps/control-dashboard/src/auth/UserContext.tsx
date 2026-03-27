import { createContext, useContext } from 'react'
import type { ReactNode } from 'react'
import type { User } from './types'

const UserContext = createContext<User | null>(null)

const MVP_USER: User = { id: 1, name: 'Default User' }

export function UserProvider({ children }: { children: ReactNode }) {
  return <UserContext.Provider value={MVP_USER}>{children}</UserContext.Provider>
}

export function useCurrentUser(): User {
  const user = useContext(UserContext)
  if (!user) {
    throw new Error('useCurrentUser must be used within a UserProvider')
  }
  return user
}
