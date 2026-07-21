import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Alert, Button, Center, Paper, Stack, TextInput, Title } from '@mantine/core'
import { api } from '../api/client'

export function Login() {
  const qc = useQueryClient()
  const [code, setCode] = useState('')
  const login = useMutation({
    mutationFn: async () => {
      const { data, error } = await api.POST('/api/v1/auth/login', { body: { code } })
      if (error) throw error
      return data
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['me'] }),
  })

  return (
    <Center h="100vh">
      <Paper withBorder p="xl" radius="md" w={360}>
        <Stack>
          <Title order={3} ta="center">
            배움마루 로그인
          </Title>
          <TextInput
            label="접속 코드"
            placeholder="XXXX-XXXX"
            value={code}
            onChange={(e) => setCode(e.currentTarget.value.toUpperCase())}
            onKeyDown={(e) => e.key === 'Enter' && code && login.mutate()}
            size="md"
          />
          {login.isError && (
            <Alert color="red">접속 코드가 올바르지 않거나 만료되었습니다.</Alert>
          )}
          <Button onClick={() => login.mutate()} loading={login.isPending} disabled={!code} size="md">
            로그인
          </Button>
        </Stack>
      </Paper>
    </Center>
  )
}
