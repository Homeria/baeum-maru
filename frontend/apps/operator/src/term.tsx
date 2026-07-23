import { createContext, useContext, useState, type ReactNode } from 'react'

const KEY = 'baeum_maru_term_id'

type TermCtx = { termId: number | null; setTermId: (id: number | null) => void }
const TermContext = createContext<TermCtx>({ termId: null, setTermId: () => {} })

export function TermProvider({ children }: { children: ReactNode }) {
  const [termId, setTermIdState] = useState<number | null>(() => {
    const saved = localStorage.getItem(KEY)
    return saved ? Number(saved) : null
  })
  const setTermId = (id: number | null) => {
    setTermIdState(id)
    if (id === null) localStorage.removeItem(KEY)
    else localStorage.setItem(KEY, String(id))
  }
  return <TermContext.Provider value={{ termId, setTermId }}>{children}</TermContext.Provider>
}

export const useTerm = () => useContext(TermContext)
