import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useForm } from '@mantine/form'
import { useNavigate } from 'react-router-dom'
import {
  Alert,
  Button,
  Checkbox,
  Group,
  Paper,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'
import { useTerm } from '../term'
import { TermNotice } from '../components/TermNotice'

type Member = components['schemas']['MemberResponse']
type Gender = Member['gender']

const GENDERS = [
  { value: 'male', label: '남' },
  { value: 'female', label: '여' },
]
const MAX_COURSES = 4

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}
function errCode(error: unknown): string | undefined {
  return (error as { error?: { code?: string } } | undefined)?.error?.code
}
async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function Enrollment() {
  const qc = useQueryClient()
  const navigate = useNavigate()
  const { termId } = useTerm()
  const [done, setDone] = useState<{ name: string; count: number } | null>(null)

  const offerings = useQuery({
    queryKey: ['offerings', termId],
    enabled: termId !== null,
    queryFn: () =>
      unwrap(api.GET('/api/v1/offerings', { params: { query: { term_id: termId ?? undefined } } })),
  })
  const courses = useQuery({ queryKey: ['courses'], queryFn: () => unwrap(api.GET('/api/v1/courses')) })

  const courseName = (id: number) => courses.data?.find((c) => c.id === id)?.name ?? id
  const openOfferings = (offerings.data ?? [])
    .filter((o) => o.status === 'open')
    .map((o) => ({
      id: String(o.id),
      label: `${courseName(o.course_id)}${o.section_label ? ` ${o.section_label}` : ''}`,
    }))

  const form = useForm({
    initialValues: {
      member_no: '',
      name: '',
      gender: 'female' as Gender,
      phone: '',
      offering_ids: [] as string[],
    },
    validate: {
      member_no: (v) => (v.trim() ? null : '회원번호를 입력하세요.'),
      name: (v) => (v.trim() ? null : '이름을 입력하세요.'),
      phone: (v) => (v.trim() ? null : '연락처를 입력하세요.'),
      offering_ids: (v) =>
        v.length < 1 ? '강좌를 1개 이상 선택하세요.' : v.length > MAX_COURSES ? `최대 ${MAX_COURSES}개까지 선택할 수 있습니다.` : null,
    },
  })

  const submit = useMutation({
    mutationFn: async (v: typeof form.values) => {
      // 1) 회원 생성. 이미 있는 회원번호면 그 회원을 재사용.
      let memberId: number
      const created = await api.POST('/api/v1/members', {
        body: { member_no: v.member_no, name: v.name, gender: v.gender, phone: v.phone },
      })
      if (created.error) {
        if (errCode(created.error) === 'member_no_exists') {
          const found = await unwrap(
            api.GET('/api/v1/members', {
              params: { query: { q: v.member_no, include_inactive: true } },
            }),
          )
          const existing = found.find((m) => m.member_no === v.member_no)
          if (!existing) throw created.error
          memberId = existing.id
        } else {
          throw created.error
        }
      } else {
        memberId = created.data.id
      }
      // 2) 선택 강좌 신청(전부 아니면 전무)
      await unwrap(
        api.POST('/api/v1/registrations', {
          body: { member_id: memberId, offering_ids: v.offering_ids.map(Number) },
        }),
      )
      return { name: v.name, count: v.offering_ids.length }
    },
    onSuccess: (result) => {
      qc.invalidateQueries({ queryKey: ['registrations'] })
      qc.invalidateQueries({ queryKey: ['members'] })
      setDone(result)
      form.reset()
    },
  })

  const selectedCount = form.values.offering_ids.length

  if (!termId) return <TermNotice />

  return (
    <Stack maw={560}>
      <Title order={4}>수강접수</Title>

      {done && (
        <Alert color="teal" withCloseButton onClose={() => setDone(null)}>
          {done.name} 님, {done.count}개 강좌 신청 완료.
        </Alert>
      )}

      <Paper withBorder p="lg">
        <form onSubmit={form.onSubmit((v) => submit.mutate(v))}>
          <Stack>
            <Group grow>
              <TextInput label="회원번호" placeholder="예: 26-00001" withAsterisk {...form.getInputProps('member_no')} />
              <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
            </Group>
            <Group grow>
              <Select label="성별" data={GENDERS} allowDeselect={false} {...form.getInputProps('gender')} />
              <TextInput label="연락처" withAsterisk {...form.getInputProps('phone')} />
            </Group>

            <Checkbox.Group
              label={`선택 강좌 (1~${MAX_COURSES}개)`}
              withAsterisk
              {...form.getInputProps('offering_ids')}
            >
              {openOfferings.length === 0 ? (
                <Alert color="yellow" mt="xs">
                  신청 받는(신청중) 개설 강좌가 없습니다.{' '}
                  <Button variant="subtle" size="compact-xs" onClick={() => navigate('/offerings')}>
                    개설 강좌로 이동
                  </Button>
                </Alert>
              ) : (
                <Stack gap="xs" mt="xs">
                  {openOfferings.map((o) => {
                    const checked = form.values.offering_ids.includes(o.id)
                    return (
                      <Checkbox
                        key={o.id}
                        value={o.id}
                        disabled={!checked && selectedCount >= MAX_COURSES}
                        label={o.label}
                      />
                    )
                  })}
                </Stack>
              )}
            </Checkbox.Group>

            {submit.isError && <Alert color="red">{errMessage(submit.error)}</Alert>}
            <Text size="xs" c="dimmed">
              이미 등록된 회원번호면 기존 회원으로 신청됩니다. 시간 충돌·인당 최대 초과 등은 저장 시 함께 검사됩니다.
            </Text>
            <Group justify="flex-end">
              <Button type="submit" loading={submit.isPending} disabled={openOfferings.length === 0}>
                접수
              </Button>
            </Group>
          </Stack>
        </form>
      </Paper>
    </Stack>
  )
}
