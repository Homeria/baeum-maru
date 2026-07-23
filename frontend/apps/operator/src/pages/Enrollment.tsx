import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useDisclosure } from '@mantine/hooks'
import { useForm } from '@mantine/form'
import { useNavigate } from 'react-router-dom'
import {
  Alert,
  Badge,
  Button,
  Card,
  Divider,
  Group,
  Modal,
  MultiSelect,
  Select,
  Stack,
  Table,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'

type Member = components['schemas']['MemberResponse']
type Gender = Member['gender']

const STATUS: Record<string, { label: string; color: string }> = {
  applied: { label: '신청', color: 'blue' },
  selected: { label: '당첨', color: 'teal' },
  waitlisted: { label: '대기', color: 'yellow' },
  confirmed: { label: '확정', color: 'green' },
  rejected: { label: '탈락', color: 'gray' },
  cancelled: { label: '취소', color: 'red' },
}
const GENDERS = [
  { value: 'male', label: '남' },
  { value: 'female', label: '여' },
]

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}
async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function Enrollment() {
  const qc = useQueryClient()
  const navigate = useNavigate()
  const [memberId, setMemberId] = useState<string | null>(null)
  const [newOpen, newModal] = useDisclosure(false)
  const [cancelReg, setCancelReg] = useState<components['schemas']['RegistrationResponse'] | null>(null)
  const [addIds, setAddIds] = useState<string[]>([])

  const members = useQuery({ queryKey: ['members'], queryFn: () => unwrap(api.GET('/api/v1/members')) })
  const offerings = useQuery({ queryKey: ['offerings', null], queryFn: () => unwrap(api.GET('/api/v1/offerings')) })
  const courses = useQuery({ queryKey: ['courses'], queryFn: () => unwrap(api.GET('/api/v1/courses')) })
  const terms = useQuery({ queryKey: ['terms'], queryFn: () => unwrap(api.GET('/api/v1/terms')) })

  const member = members.data?.find((m) => String(m.id) === memberId) ?? null
  const courseName = (id: number) => courses.data?.find((c) => c.id === id)?.name ?? id
  const offeringLabel = (id: number) => {
    const o = offerings.data?.find((x) => x.id === id)
    return o ? `${courseName(o.course_id)}${o.section_label ? ` ${o.section_label}` : ''}` : String(id)
  }

  const regs = useQuery({
    queryKey: ['registrations', memberId],
    enabled: member !== null,
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/registrations', {
          params: { query: { member_id: Number(memberId) } },
        }),
      ),
  })

  // 신규 회원 등록 → 등록 후 그 회원으로 바로 선택
  const newForm = useForm({
    initialValues: { member_no: '', name: '', gender: 'female' as Gender, phone: '' },
    validate: {
      member_no: (v) => (v.trim() ? null : '회원번호를 입력하세요.'),
      name: (v) => (v.trim() ? null : '이름을 입력하세요.'),
      phone: (v) => (v.trim() ? null : '연락처를 입력하세요.'),
    },
  })
  const createMember = useMutation({
    mutationFn: (v: typeof newForm.values) => unwrap(api.POST('/api/v1/members', { body: v })),
    onSuccess: (created) => {
      qc.invalidateQueries({ queryKey: ['members'] })
      setMemberId(String(created.id))
      newModal.close()
      newForm.reset()
    },
  })

  const openOfferings = (offerings.data ?? [])
    .filter((o) => o.status === 'open')
    .map((o) => {
      const t = terms.data?.find((x) => x.id === o.term_id)
      return { value: String(o.id), label: `${offeringLabel(o.id)} · ${t?.name ?? o.term_id}` }
    })

  const addRegs = useMutation({
    mutationFn: () =>
      unwrap(
        api.POST('/api/v1/registrations', {
          body: { member_id: Number(memberId), offering_ids: addIds.map(Number) },
        }),
      ),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['registrations', memberId] })
      setAddIds([])
    },
  })

  const cancel = useMutation({
    mutationFn: (reason: string) =>
      unwrap(
        api.POST('/api/v1/registrations/{registration_id}/cancel', {
          params: { path: { registration_id: cancelReg!.id } },
          body: { reason: reason || null },
        }),
      ),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['registrations', memberId] })
      setCancelReg(null)
    },
  })
  const cancelForm = useForm({ initialValues: { reason: '' } })

  return (
    <Stack>
      <Title order={4}>수강접수</Title>

      <Group align="flex-end">
        <Select
          label="회원"
          placeholder="이름·회원번호로 검색"
          searchable
          w={320}
          data={(members.data ?? []).map((m) => ({
            value: String(m.id),
            label: `${m.name} (${m.member_no})${m.is_active ? '' : ' · 비활성'}`,
          }))}
          value={memberId}
          onChange={setMemberId}
        />
        <Button variant="light" onClick={newModal.open}>
          신규 회원 등록
        </Button>
      </Group>

      {member && (
        <Card withBorder>
          <Group justify="space-between">
            <div>
              <Text fw={600} size="lg">
                {member.name}{' '}
                <Text span c="dimmed" size="sm">
                  {member.member_no}
                </Text>
              </Text>
              <Text size="sm" c="dimmed">
                {member.gender === 'male' ? '남' : '여'} · {member.phone}
                {!member.is_active && ' · 비활성'}
              </Text>
            </div>
          </Group>

          <Divider my="md" label="신청 현황" labelPosition="left" />
          <Table>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>강좌</Table.Th>
                <Table.Th>상태</Table.Th>
                <Table.Th>대기순번</Table.Th>
                <Table.Th />
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {regs.data?.map((r) => {
                const info = STATUS[r.status] ?? { label: r.status, color: 'gray' }
                return (
                  <Table.Tr key={r.id}>
                    <Table.Td>{offeringLabel(r.offering_id)}</Table.Td>
                    <Table.Td>
                      <Badge color={info.color}>{info.label}</Badge>
                    </Table.Td>
                    <Table.Td>{r.waitlist_order ?? '-'}</Table.Td>
                    <Table.Td>
                      {r.status !== 'cancelled' && (
                        <Button size="xs" variant="subtle" color="red" onClick={() => setCancelReg(r)}>
                          취소
                        </Button>
                      )}
                    </Table.Td>
                  </Table.Tr>
                )
              })}
            </Table.Tbody>
          </Table>
          {regs.data?.length === 0 && (
            <Text c="dimmed" size="sm" ta="center" py="sm">
              신청 내역이 없습니다.
            </Text>
          )}

          <Divider my="md" label="강좌 추가" labelPosition="left" />
          {openOfferings.length === 0 ? (
            <Alert color="yellow">
              신청 받는(신청중) 개설 강좌가 없습니다.{' '}
              <Button variant="subtle" size="compact-xs" onClick={() => navigate('/offerings')}>
                개설 강좌로 이동
              </Button>
            </Alert>
          ) : (
            <Group align="flex-end">
              <MultiSelect
                flex={1}
                placeholder="추가할 강좌 선택"
                searchable
                data={openOfferings}
                value={addIds}
                onChange={setAddIds}
              />
              <Button
                onClick={() => addRegs.mutate()}
                loading={addRegs.isPending}
                disabled={!addIds.length}
              >
                신청
              </Button>
            </Group>
          )}
          {addRegs.isError && <Alert color="red" mt="xs">{errMessage(addRegs.error)}</Alert>}
        </Card>
      )}

      {/* 신규 회원 */}
      <Modal opened={newOpen} onClose={newModal.close} title="신규 회원 등록">
        <form onSubmit={newForm.onSubmit((v) => createMember.mutate(v))}>
          <Stack>
            <TextInput label="회원번호" placeholder="예: 26-00001" withAsterisk {...newForm.getInputProps('member_no')} />
            <TextInput label="이름" withAsterisk {...newForm.getInputProps('name')} />
            <Select label="성별" data={GENDERS} allowDeselect={false} {...newForm.getInputProps('gender')} />
            <TextInput label="연락처" withAsterisk {...newForm.getInputProps('phone')} />
            {createMember.isError && <Alert color="red">{errMessage(createMember.error)}</Alert>}
            <Group justify="flex-end">
              <Button variant="default" onClick={newModal.close}>
                취소
              </Button>
              <Button type="submit" loading={createMember.isPending}>
                등록
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>

      {/* 취소 */}
      <Modal opened={cancelReg !== null} onClose={() => setCancelReg(null)} title="수강 취소">
        <form onSubmit={cancelForm.onSubmit((v) => cancel.mutate(v.reason))}>
          <Stack>
            <Text size="sm">{cancelReg && offeringLabel(cancelReg.offering_id)} 신청을 취소합니다.</Text>
            <Text size="xs" c="dimmed">
              당첨자를 취소하면 대기 순번대로 자동 승계됩니다.
            </Text>
            <TextInput label="사유(선택)" {...cancelForm.getInputProps('reason')} />
            {cancel.isError && <Alert color="red">{errMessage(cancel.error)}</Alert>}
            <Group justify="flex-end">
              <Button variant="default" onClick={() => setCancelReg(null)}>
                닫기
              </Button>
              <Button type="submit" color="red" loading={cancel.isPending}>
                취소 처리
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>
    </Stack>
  )
}
