import { Center, Loader } from '@mantine/core'
import { Navigate, Route, Routes } from 'react-router-dom'
import { useAuth } from './auth'
import { Layout } from './Layout'
import { Login } from './pages/Login'
import { Enrollment } from './pages/Enrollment'
import { Members } from './pages/Members'
import { Catalog } from './pages/Catalog'
import { Terms } from './pages/Terms'
import { Offerings } from './pages/Offerings'
import { Spaces } from './pages/Spaces'
import { Registrations } from './pages/Registrations'
import { Lottery } from './pages/Lottery'

export default function App() {
  const { operator, loading } = useAuth()
  if (loading)
    return (
      <Center h="100vh">
        <Loader />
      </Center>
    )
  if (!operator) return <Login />
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/enrollment" element={<Enrollment />} />
        <Route path="/registrations" element={<Registrations />} />
        <Route path="/offerings" element={<Offerings />} />
        <Route path="/lottery" element={<Lottery />} />
        <Route path="/members" element={<Members />} />
        <Route path="/catalog" element={<Catalog />} />
        <Route path="/spaces" element={<Spaces />} />
        <Route path="/terms" element={<Terms />} />
        <Route path="*" element={<Navigate to="/enrollment" replace />} />
      </Route>
    </Routes>
  )
}
