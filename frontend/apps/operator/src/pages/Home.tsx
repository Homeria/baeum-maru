import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Button, Container, Divider, Group, Text, Title } from '@mantine/core'
import { api } from '../api/client'
import { useAuth } from '../auth'
import { Operators } from './Operators'

export function Home() {
  const { operator } = useAuth()
  const qc = useQueryClient()
  const logout = useMutation({
    mutationFn: async () => {
      await api.POST('/api/v1/auth/logout')
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['me'] }),
  })

  return (
    <Container py="lg">
      <Group justify="space-between">
        <Title order={3}>배움마루</Title>
        <Group>
          <Text c="dimmed">
            {operator?.display_name} · {operator?.role}
          </Text>
          <Button variant="light" onClick={() => logout.mutate()} loading={logout.isPending}>
            로그아웃
          </Button>
        </Group>
      </Group>
      <Divider my="lg" />
      <Operators />
    </Container>
  )
}
