import { AppShell, Burger, Button, Group, NavLink, Select, Text, Title } from '@mantine/core'
import { useDisclosure } from '@mantine/hooks'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from './api/client'
import { useAuth } from './auth'
import { useTerm } from './term'

const GROUPS = [
  {
    title: '학기 업무',
    items: [
      { to: '/enrollment', label: '수강접수' },
      { to: '/registrations', label: '신청 현황' },
      { to: '/offerings', label: '개설 강좌' },
      { to: '/lottery', label: '강좌 추첨' },
    ],
  },
  {
    title: '설정',
    items: [
      { to: '/members', label: '회원' },
      { to: '/catalog', label: '과목·강사' },
      { to: '/spaces', label: '공간' },
      { to: '/terms', label: '학기 관리' },
    ],
  },
]

export function Layout() {
  const [opened, { toggle }] = useDisclosure()
  const { operator } = useAuth()
  const { termId, setTermId } = useTerm()
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const qc = useQueryClient()

  const terms = useQuery({
    queryKey: ['terms'],
    queryFn: async () => {
      const { data, error } = await api.GET('/api/v1/terms')
      if (error) throw error
      return data
    },
  })

  const logout = useMutation({
    mutationFn: async () => {
      await api.POST('/api/v1/auth/logout')
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['me'] }),
  })

  return (
    <AppShell
      header={{ height: 56 }}
      navbar={{ width: 210, breakpoint: 'sm', collapsed: { mobile: !opened } }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md" justify="space-between">
          <Group>
            <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
            <Title order={4}>배움마루</Title>
            <Select
              placeholder="학기 선택"
              w={180}
              data={(terms.data ?? []).map((t) => ({ value: String(t.id), label: t.name }))}
              value={termId ? String(termId) : null}
              onChange={(v) => setTermId(v ? Number(v) : null)}
              nothingFoundMessage="학기 없음 · 학기 관리에서 추가"
              searchable
            />
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
        {GROUPS.map((g) => (
          <div key={g.title}>
            <Text size="xs" c="dimmed" fw={600} tt="uppercase" px="xs" mt="sm" mb={4}>
              {g.title}
            </Text>
            {g.items.map((n) => (
              <NavLink
                key={n.to}
                label={n.label}
                active={pathname === n.to}
                onClick={() => navigate(n.to)}
              />
            ))}
          </div>
        ))}
      </AppShell.Navbar>
      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  )
}
