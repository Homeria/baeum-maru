import { AppShell, Burger, Button, Group, NavLink, Text, Title } from '@mantine/core'
import { useDisclosure } from '@mantine/hooks'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { api } from './api/client'
import { useAuth } from './auth'

const NAV = [
  { to: '/operators', label: '관계자' },
  { to: '/members', label: '회원' },
]

export function Layout() {
  const [opened, { toggle }] = useDisclosure()
  const { operator } = useAuth()
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const qc = useQueryClient()
  const logout = useMutation({
    mutationFn: async () => {
      await api.POST('/api/v1/auth/logout')
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['me'] }),
  })

  return (
    <AppShell
      header={{ height: 56 }}
      navbar={{ width: 200, breakpoint: 'sm', collapsed: { mobile: !opened } }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md" justify="space-between">
          <Group>
            <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
            <Title order={4}>배움마루</Title>
          </Group>
          <Group>
            <Text c="dimmed" size="sm">
              {operator?.display_name}
            </Text>
            <Button size="xs" variant="light" onClick={() => logout.mutate()}>
              로그아웃
            </Button>
          </Group>
        </Group>
      </AppShell.Header>
      <AppShell.Navbar p="sm">
        {NAV.map((n) => (
          <NavLink
            key={n.to}
            label={n.label}
            active={pathname === n.to}
            onClick={() => navigate(n.to)}
          />
        ))}
      </AppShell.Navbar>
      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  )
}
