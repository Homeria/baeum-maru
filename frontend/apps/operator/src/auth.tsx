import { createContext, useContext, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from './api/client'

export type Operator = { id: number; display_name: string; role: string }

const AuthContext = createContext<{ operator: Operator | null; loading: boolean }>({
  operator: null,
  loading: true,
})

export function AuthProvider({ children }: { children: ReactNode }) {
  // /auth/me: 미로그인이면 401 → null. 이 쿼리 하나가 로그인 상태의 단일 출처.
  const { data, isLoading } = useQuery({
    queryKey: ['me'],
    queryFn: async () => {
      const { data, error } = await api.GET('/api/v1/auth/me')
      return error ? null : data
    },
    retry: false,
  })
  return (
    <AuthContext.Provider value={{ operator: data ?? null, loading: isLoading }}>
      {children}
    </AuthContext.Provider>
  )
}

export const useAuth = () => useContext(AuthContext)
