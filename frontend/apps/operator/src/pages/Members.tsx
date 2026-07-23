import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useDisclosure } from '@mantine/hooks'
import { useForm } from '@mantine/form'
import {
  Alert,
  Badge,
  Button,
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

type Member = components['schemas']['MemberResponse']
type Gender = Member['gender']

const GENDERS = [
  { value: 'male', label: '남' },
  { value: 'female', label: '여' },
]
const genderLabel = (g: string) => GENDERS.find((x) => x.value === g)?.label ?? g

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}

export function Members() {
  const qc = useQueryClient()
  const [q, setQ] = useState('')
  const [includeInactive, setIncludeInactive] = useState(false)
  const [editing, setEditing] = useState<Member | null>(null)
  const [deleting, setDeleting] = useState<Member | null>(null)
  const [formOpen, form] = useDisclosure(false)

  const list = useQuery({
    queryKey: ['members', q, includeInactive],
    queryFn: async () => {
      const { data, error } = await api.GET('/api/v1/members', {
        params: { query: { q: q || undefined, include_inactive: includeInactive } },
      })
      if (error) throw error
      return data
    },
  })

  const formHook = useForm({
    initialValues: { member_no: '', name: '', gender: 'female' as Gender, phone: '', is_active: true },
    validate: {
      member_no: (v) => (v.trim() ? null : '회원번호를 입력하세요.'),
      name: (v) => (v.trim() ? null : '이름을 입력하세요.'),
      phone: (v) => (v.trim() ? null : '연락처를 입력하세요.'),
    },
  })

  const openCreate = () => {
    setEditing(null)
    formHook.setValues({ member_no: '', name: '', gender: 'female', phone: '', is_active: true })
    form.open()
  }
  const openEdit = (m: Member) => {
    setEditing(m)
    formHook.setValues({
      member_no: m.member_no,
      name: m.name,
      gender: m.gender,
      phone: m.phone,
      is_active: m.is_active,
    })
    form.open()
  }

  const save = useMutation({
    mutationFn: async (v: typeof formHook.values) => {
      if (editing) {
        const { error } = await api.PATCH('/api/v1/members/{member_id}', {
          params: { path: { member_id: editing.id } },
          body: { name: v.name, gender: v.gender, phone: v.phone, is_active: v.is_active },
        })
        if (error) throw error
      } else {
        const { error } = await api.POST('/api/v1/members', {
          body: { member_no: v.member_no, name: v.name, gender: v.gender, phone: v.phone },
        })
        if (error) throw error
      }
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['members'] })
      form.close()
    },
  })

  const remove = useMutation({
    mutationFn: async (m: Member) => {
      const { error } = await api.DELETE('/api/v1/members/{member_id}', {
        params: { path: { member_id: m.id } },
      })
      if (error) throw error
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['members'] })
      setDeleting(null)
    },
  })

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={4}>회원</Title>
        <Group>
          <TextInput
            placeholder="이름·연락처·회원번호 검색"
            value={q}
            onChange={(e) => setQ(e.currentTarget.value)}
            w={240}
          />
          <Switch
            label="비활성 포함"
            checked={includeInactive}
            onChange={(e) => setIncludeInactive(e.currentTarget.checked)}
          />
          <Button onClick={openCreate}>회원 등록</Button>
        </Group>
      </Group>

      {list.isError && <Alert color="red">{errMessage(list.error)}</Alert>}

      <Table striped highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>회원번호</Table.Th>
            <Table.Th>이름</Table.Th>
            <Table.Th>성별</Table.Th>
            <Table.Th>연락처</Table.Th>
            <Table.Th>상태</Table.Th>
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {list.data?.map((m) => (
            <Table.Tr key={m.id}>
              <Table.Td>{m.member_no}</Table.Td>
              <Table.Td>{m.name}</Table.Td>
              <Table.Td>{genderLabel(m.gender)}</Table.Td>
              <Table.Td>{m.phone}</Table.Td>
              <Table.Td>
                <Badge color={m.is_active ? 'teal' : 'gray'}>
                  {m.is_active ? '활성' : '비활성'}
                </Badge>
              </Table.Td>
              <Table.Td>
                <Group gap="xs" justify="flex-end">
                  <Button size="xs" variant="subtle" onClick={() => openEdit(m)}>
                    수정
                  </Button>
                  <Button size="xs" variant="subtle" color="red" onClick={() => setDeleting(m)}>
                    삭제
                  </Button>
                </Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>

      <Modal opened={formOpen} onClose={form.close} title={editing ? '회원 수정' : '회원 등록'}>
        <form onSubmit={formHook.onSubmit((v) => save.mutate(v))}>
          <Stack>
            <TextInput
              label="회원번호"
              placeholder="예: 26-00001"
              withAsterisk
              disabled={editing !== null}
              {...formHook.getInputProps('member_no')}
            />
            <TextInput label="이름" withAsterisk {...formHook.getInputProps('name')} />
            <Select
              label="성별"
              data={GENDERS}
              allowDeselect={false}
              {...formHook.getInputProps('gender')}
            />
            <TextInput label="연락처" withAsterisk {...formHook.getInputProps('phone')} />
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

      <Modal opened={deleting !== null} onClose={() => setDeleting(null)} title="회원 삭제">
        <Stack>
          <Text size="sm">
            {deleting && `${deleting.name} (${deleting.member_no})`} 회원을 삭제할까요?
          </Text>
          <Text size="xs" c="dimmed">
            신청 이력이 있으면 삭제되지 않습니다. 그때는 비활성화하세요.
          </Text>
          {remove.isError && <Alert color="red">{errMessage(remove.error)}</Alert>}
          <Group justify="flex-end">
            <Button variant="default" onClick={() => setDeleting(null)}>
              취소
            </Button>
            <Button color="red" loading={remove.isPending} onClick={() => deleting && remove.mutate(deleting)}>
              삭제
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Stack>
  )
}
