import { Center, Loader } from '@mantine/core'
import { Navigate, Route, Routes } from 'react-router-dom'
import { useAuth } from './auth'
import { Layout } from './Layout'
import { Login } from './pages/Login'
import { Enrollment } from './pages/Enrollment'
import { Members } from './pages/Members'
import { CourseMasters } from './pages/CourseMasters'
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
        <Route path="/lottery" element={<Lottery />} />
        <Route path="/members" element={<Members />} />
        <Route path="/offerings" element={<Offerings />} />
        <Route path="/course-masters" element={<CourseMasters />} />
        <Route path="/spaces" element={<Spaces />} />
        <Route path="*" element={<Navigate to="/enrollment" replace />} />
      </Route>
    </Routes>
  )
}
