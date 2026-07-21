import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Button, Container, Group, Text, Title } from '@mantine/core'
import { api } from '../api/client'
import { useAuth } from '../auth'

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
      <Text mt="xl">로그인됨. 업무 화면은 다음 슬라이스에서 도메인별로 추가합니다.</Text>
    </Container>
  )
}
