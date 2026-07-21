import { Center, Loader } from '@mantine/core'
import { Navigate, Route, Routes } from 'react-router-dom'
import { useAuth } from './auth'
import { Layout } from './Layout'
import { Login } from './pages/Login'
import { Operators } from './pages/Operators'
import { Members } from './pages/Members'
import { CourseMasters } from './pages/CourseMasters'

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
        <Route path="/operators" element={<Operators />} />
        <Route path="/members" element={<Members />} />
        <Route path="/course-masters" element={<CourseMasters />} />
        <Route path="*" element={<Navigate to="/operators" replace />} />
      </Route>
    </Routes>
  )
}
