import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useDisclosure } from '@mantine/hooks'
import { useForm } from '@mantine/form'
import {
  Alert,
  Badge,
  Button,
  CopyButton,
  Group,
  Modal,
  Select,
  Stack,
  Switch,
  Table,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'

type Operator = components['schemas']['OperatorResponse']
type AccessCode = components['schemas']['AccessCodeResponse']

const ROLES = [
  { value: 'staff', label: '직원' },
  { value: 'temporary_staff', label: '임시직원' },
  { value: 'viewer', label: '열람전용' },
]
const roleLabel = (r: string) => ROLES.find((x) => x.value === r)?.label ?? r

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}

export function Operators() {
  const qc = useQueryClient()
  const [includeInactive, setIncludeInactive] = useState(false)
  const [editing, setEditing] = useState<Operator | null>(null)
  const [issuedCode, setIssuedCode] = useState<string | null>(null)
  const [codesOp, setCodesOp] = useState<Operator | null>(null)
  const [formOpen, form] = useDisclosure(false)

  const list = useQuery({
    queryKey: ['operators', includeInactive],
    queryFn: async () => {
      const { data, error } = await api.GET('/api/v1/operators', {
        params: { query: { include_inactive: includeInactive } },
      })
      if (error) throw error
      return data
    },
  })

  const formHook = useForm({
    initialValues: { display_name: '', role: 'staff', is_active: true },
    validate: { display_name: (v) => (v.trim() ? null : '이름을 입력하세요.') },
  })

  const openCreate = () => {
    setEditing(null)
    formHook.setValues({ display_name: '', role: 'staff', is_active: true })
    form.open()
  }
  const openEdit = (op: Operator) => {
    setEditing(op)
    formHook.setValues({ display_name: op.display_name, role: op.role, is_active: op.is_active })
    form.open()
  }

  const save = useMutation({
    mutationFn: async (v: typeof formHook.values) => {
      if (editing) {
        const { error } = await api.PATCH('/api/v1/operators/{operator_id}', {
          params: { path: { operator_id: editing.id } },
          body: { display_name: v.display_name, role: v.role as Operator['role'], is_active: v.is_active },
        })
        if (error) throw error
      } else {
        const { error } = await api.POST('/api/v1/operators', {
          body: { display_name: v.display_name, role: v.role as Operator['role'] },
        })
        if (error) throw error
      }
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['operators'] })
      form.close()
    },
  })

  const issue = useMutation({
    mutationFn: async (op: Operator) => {
      const { data, error } = await api.POST('/api/v1/operators/{operator_id}/access-codes', {
        params: { path: { operator_id: op.id } },
        body: {},
      })
      if (error) throw error
      return data.code
    },
    onSuccess: (code) => setIssuedCode(code),
  })

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={4}>관계자</Title>
        <Group>
          <Switch
            label="비활성 포함"
            checked={includeInactive}
            onChange={(e) => setIncludeInactive(e.currentTarget.checked)}
          />
          <Button onClick={openCreate}>관계자 추가</Button>
        </Group>
      </Group>

      {list.isError && <Alert color="red">{errMessage(list.error)}</Alert>}

      <Table striped highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>이름</Table.Th>
            <Table.Th>역할</Table.Th>
            <Table.Th>상태</Table.Th>
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {list.data?.map((op) => (
            <Table.Tr key={op.id}>
              <Table.Td>{op.display_name}</Table.Td>
              <Table.Td>{roleLabel(op.role)}</Table.Td>
              <Table.Td>
                <Badge color={op.is_active ? 'teal' : 'gray'}>
                  {op.is_active ? '활성' : '비활성'}
                </Badge>
              </Table.Td>
              <Table.Td>
                <Group gap="xs" justify="flex-end">
                  <Button
                    size="xs"
                    variant="light"
                    disabled={!op.is_active}
                    loading={issue.isPending && issue.variables?.id === op.id}
                    onClick={() => issue.mutate(op)}
                  >
                    접속코드 발급
                  </Button>
                  <Button size="xs" variant="subtle" onClick={() => setCodesOp(op)}>
                    코드 목록
                  </Button>
                  <Button size="xs" variant="subtle" onClick={() => openEdit(op)}>
                    수정
                  </Button>
                </Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>

      {/* 생성/수정 폼 */}
      <Modal opened={formOpen} onClose={form.close} title={editing ? '관계자 수정' : '관계자 추가'}>
        <form onSubmit={formHook.onSubmit((v) => save.mutate(v))}>
          <Stack>
            <TextInput label="이름" withAsterisk {...formHook.getInputProps('display_name')} />
            <Select label="역할" data={ROLES} allowDeselect={false} {...formHook.getInputProps('role')} />
            {editing && (
              <Switch
                label="활성"
                checked={formHook.values.is_active}
                onChange={(e) => formHook.setFieldValue('is_active', e.currentTarget.checked)}
              />
            )}
            {save.isError && <Alert color="red">{errMessage(save.error)}</Alert>}
            <Group justify="flex-end">
              <Button variant="default" onClick={form.close}>
                취소
              </Button>
              <Button type="submit" loading={save.isPending}>
                저장
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>

      {/* 발급된 코드 1회 노출 */}
      <Modal opened={issuedCode !== null} onClose={() => setIssuedCode(null)} title="접속 코드 발급됨">
        <Stack>
          <Text size="sm" c="dimmed">
            이 코드는 지금 한 번만 표시됩니다. 관계자에게 전달하세요.
          </Text>
          <Group>
            <Text fw={700} size="xl" ff="monospace">
              {issuedCode}
            </Text>
            <CopyButton value={issuedCode ?? ''}>
              {({ copied, copy }) => (
                <Button size="xs" variant="light" onClick={copy}>
                  {copied ? '복사됨' : '복사'}
                </Button>
              )}
            </CopyButton>
          </Group>
        </Stack>
      </Modal>

      {/* 코드 목록/폐기 */}
      <CodesModal operator={codesOp} onClose={() => setCodesOp(null)} />
    </Stack>
  )
}

function CodesModal({ operator, onClose }: { operator: Operator | null; onClose: () => void }) {
  const qc = useQueryClient()
  const codes = useQuery({
    queryKey: ['codes', operator?.id],
    enabled: operator !== null,
    queryFn: async () => {
      const { data, error } = await api.GET('/api/v1/operators/{operator_id}/access-codes', {
        params: { path: { operator_id: operator!.id } },
      })
      if (error) throw error
      return data
    },
  })

  const revoke = useMutation({
    mutationFn: async (code: AccessCode) => {
      const { error } = await api.POST(
        '/api/v1/operators/{operator_id}/access-codes/{code_id}/revoke',
        { params: { path: { operator_id: operator!.id, code_id: code.id } } },
      )
      if (error) throw error
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['codes', operator?.id] }),
  })

  const isActive = (c: AccessCode) =>
    !c.revoked_at && new Date(c.expires_at.replace(' ', 'T') + 'Z') > new Date()

  return (
    <Modal opened={operator !== null} onClose={onClose} title={`${operator?.display_name} 접속 코드`} size="lg">
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>발급</Table.Th>
            <Table.Th>만료</Table.Th>
            <Table.Th>상태</Table.Th>
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {codes.data?.map((c) => (
            <Table.Tr key={c.id}>
              <Table.Td>{c.issued_at}</Table.Td>
              <Table.Td>{c.expires_at}</Table.Td>
              <Table.Td>
                <Badge color={c.revoked_at ? 'gray' : isActive(c) ? 'teal' : 'orange'}>
                  {c.revoked_at ? '폐기됨' : isActive(c) ? '유효' : '만료'}
                </Badge>
              </Table.Td>
              <Table.Td>
                {isActive(c) && (
                  <Button
                    size="xs"
                    color="red"
                    variant="light"
                    loading={revoke.isPending && revoke.variables?.id === c.id}
                    onClick={() => revoke.mutate(c)}
                  >
                    폐기
                  </Button>
                )}
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
      {codes.data?.length === 0 && (
        <Text c="dimmed" ta="center" py="md">
          발급된 코드가 없습니다.
        </Text>
      )}
    </Modal>
  )
}
