import { Center, Loader } from '@mantine/core'
import { useAuth } from './auth'
import { Login } from './pages/Login'
import { Home } from './pages/Home'

export default function App() {
  const { operator, loading } = useAuth()
  if (loading)
    return (
      <Center h="100vh">
        <Loader />
      </Center>
    )
  return operator ? <Home /> : <Login />
}
