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

type Registration = components['schemas']['RegistrationResponse']
type History = components['schemas']['StatusHistoryResponse']

const STATUS = [
  { value: 'applied', label: '신청', color: 'blue' },
  { value: 'selected', label: '당첨', color: 'teal' },
  { value: 'waitlisted', label: '대기', color: 'yellow' },
  { value: 'confirmed', label: '확정', color: 'green' },
  { value: 'rejected', label: '탈락', color: 'gray' },
  { value: 'cancelled', label: '취소', color: 'red' },
]
const statusInfo = (s: string) => STATUS.find((x) => x.value === s) ?? { label: s, color: 'gray' }

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}
async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function Registrations() {
  const qc = useQueryClient()
  const [term, setTerm] = useState<string | null>(null)
  const [offering, setOffering] = useState<string | null>(null)
  const [status, setStatus] = useState<string | null>(null)
  const [applyOpen, apply] = useDisclosure(false)
  const [cancelReg, setCancelReg] = useState<Registration | null>(null)
  const [historyReg, setHistoryReg] = useState<Registration | null>(null)

  const members = useQuery({ queryKey: ['members'], queryFn: () => unwrap(api.GET('/api/v1/members')) })
  const offerings = useQuery({ queryKey: ['offerings', null], queryFn: () => unwrap(api.GET('/api/v1/offerings')) })
  const courses = useQuery({ queryKey: ['courses'], queryFn: () => unwrap(api.GET('/api/v1/courses')) })
  const terms = useQuery({ queryKey: ['terms'], queryFn: () => unwrap(api.GET('/api/v1/terms')) })

  const courseName = (id: number) => courses.data?.find((c) => c.id === id)?.name ?? id
  const memberName = (id: number) => {
    const m = members.data?.find((x) => x.id === id)
    return m ? `${m.name} (${m.member_no})` : String(id)
  }
  const offeringLabel = (id: number) => {
    const o = offerings.data?.find((x) => x.id === id)
    if (!o) return String(id)
    return `${courseName(o.course_id)}${o.section_label ? ` ${o.section_label}` : ''}`
  }

  const list = useQuery({
    queryKey: ['registrations', term, offering, status],
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/registrations', {
          params: {
            query: {
              term_id: term ? Number(term) : undefined,
              offering_id: offering ? Number(offering) : undefined,
              status: status || undefined,
            },
          },
        }),
      ),
  })

  // 필터 학기에 속한 개설만 offering 필터 옵션으로
  const offeringOptions = (offerings.data ?? [])
    .filter((o) => !term || o.term_id === Number(term))
    .map((o) => ({ value: String(o.id), label: offeringLabel(o.id) }))

  const applyForm = useForm<{ member_id: string; offering_ids: string[] }>({
    initialValues: { member_id: '', offering_ids: [] },
    validate: {
      member_id: (v) => (v ? null : '회원을 선택하세요.'),
      offering_ids: (v) => (v.length ? null : '강좌를 하나 이상 선택하세요.'),
    },
  })
  const doApply = useMutation({
    mutationFn: async (v: typeof applyForm.values) => {
      await unwrap(
        api.POST('/api/v1/registrations', {
          body: { member_id: Number(v.member_id), offering_ids: v.offering_ids.map(Number) },
        }),
      )
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['registrations'] })
      apply.close()
      applyForm.reset()
    },
  })

  const cancelForm = useForm({ initialValues: { reason: '' } })
  const doCancel = useMutation({
    mutationFn: async (v: { reason: string }) => {
      await unwrap(
        api.POST('/api/v1/registrations/{registration_id}/cancel', {
          params: { path: { registration_id: cancelReg!.id } },
          body: { reason: v.reason || null },
        }),
      )
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['registrations'] })
      setCancelReg(null)
      cancelForm.reset()
    },
  })

  const openApplyOfferings = (offerings.data ?? [])
    .filter((o) => o.status === 'open')
    .map((o) => {
      const t = terms.data?.find((x) => x.id === o.term_id)
      return { value: String(o.id), label: `${offeringLabel(o.id)} · ${t?.name ?? o.term_id}` }
    })

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={4}>수강신청</Title>
        <Group>
          <Select
            placeholder="학기"
            clearable
            w={150}
            data={(terms.data ?? []).map((t) => ({ value: String(t.id), label: t.name }))}
            value={term}
            onChange={(v) => {
              setTerm(v)
              setOffering(null)
            }}
          />
          <Select
            placeholder="개설강좌"
            clearable
            searchable
            w={200}
            data={offeringOptions}
            value={offering}
            onChange={setOffering}
          />
          <Select
            placeholder="상태"
            clearable
            w={120}
            data={STATUS.map((s) => ({ value: s.value, label: s.label }))}
            value={status}
            onChange={setStatus}
          />
          <Button onClick={apply.open}>수강신청</Button>
        </Group>
      </Group>

      {list.isError && <Alert color="red">{errMessage(list.error)}</Alert>}

      <Table striped highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>회원</Table.Th>
            <Table.Th>강좌</Table.Th>
            <Table.Th>상태</Table.Th>
            <Table.Th>대기순번</Table.Th>
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {list.data?.map((r) => {
            const info = statusInfo(r.status)
            return (
              <Table.Tr key={r.id}>
                <Table.Td>{memberName(r.member_id)}</Table.Td>
                <Table.Td>{offeringLabel(r.offering_id)}</Table.Td>
                <Table.Td>
                  <Badge color={info.color}>{info.label}</Badge>
                </Table.Td>
                <Table.Td>{r.waitlist_order ?? '-'}</Table.Td>
                <Table.Td>
                  <Group gap="xs" justify="flex-end">
                    <Button size="xs" variant="subtle" onClick={() => setHistoryReg(r)}>
                      이력
                    </Button>
                    {r.status !== 'cancelled' && (
                      <Button size="xs" variant="light" color="red" onClick={() => setCancelReg(r)}>
                        취소
                      </Button>
                    )}
                  </Group>
                </Table.Td>
              </Table.Tr>
            )
          })}
        </Table.Tbody>
      </Table>

      {/* 신청 */}
      <Modal opened={applyOpen} onClose={apply.close} title="수강신청">
        <form onSubmit={applyForm.onSubmit((v) => doApply.mutate(v))}>
          <Stack>
            <Select
              label="회원"
              withAsterisk
              searchable
              data={(members.data ?? [])
                .filter((m) => m.is_active)
                .map((m) => ({ value: String(m.id), label: `${m.name} (${m.member_no})` }))}
              {...applyForm.getInputProps('member_id')}
            />
            <MultiSelect
              label="개설 강좌 (신청중)"
              withAsterisk
              searchable
              data={openApplyOfferings}
              {...applyForm.getInputProps('offering_ids')}
            />
            {doApply.isError && <Alert color="red">{errMessage(doApply.error)}</Alert>}
            <Text size="xs" c="dimmed">
              시간 충돌·인당 최대 초과 등은 저장 시 함께 검사됩니다(전부 아니면 전무).
            </Text>
            <Group justify="flex-end">
              <Button variant="default" onClick={apply.close}>
                취소
              </Button>
              <Button type="submit" loading={doApply.isPending}>
                신청
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>

      {/* 취소 */}
      <Modal opened={cancelReg !== null} onClose={() => setCancelReg(null)} title="수강 취소">
        <form onSubmit={cancelForm.onSubmit((v) => doCancel.mutate(v))}>
          <Stack>
            <Text size="sm">
              {cancelReg && `${memberName(cancelReg.member_id)} · ${offeringLabel(cancelReg.offering_id)}`}
            </Text>
            <Text size="xs" c="dimmed">
              당첨자를 취소하면 대기 순번대로 자동 승계됩니다.
            </Text>
            <TextInput label="사유(선택)" {...cancelForm.getInputProps('reason')} />
            {doCancel.isError && <Alert color="red">{errMessage(doCancel.error)}</Alert>}
            <Group justify="flex-end">
              <Button variant="default" onClick={() => setCancelReg(null)}>
                닫기
              </Button>
              <Button type="submit" color="red" loading={doCancel.isPending}>
                취소 처리
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>

      {/* 이력 */}
      <HistoryModal registration={historyReg} onClose={() => setHistoryReg(null)} />
    </Stack>
  )
}

function HistoryModal({
  registration,
  onClose,
}: {
  registration: Registration | null
  onClose: () => void
}) {
  const history = useQuery({
    queryKey: ['reg-history', registration?.id],
    enabled: registration !== null,
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/registrations/{registration_id}/history', {
          params: { path: { registration_id: registration!.id } },
        }),
      ),
  })
  const lbl = (s: string | null) => (s ? statusInfo(s).label : '-')

  return (
    <Modal opened={registration !== null} onClose={onClose} title="상태 이력" size="lg">
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>시각</Table.Th>
            <Table.Th>변경</Table.Th>
            <Table.Th>사유</Table.Th>
            <Table.Th>행위자</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {history.data?.map((h: History) => (
            <Table.Tr key={h.id}>
              <Table.Td>{h.changed_at}</Table.Td>
              <Table.Td>
                {lbl(h.from_status)} → {lbl(h.to_status)}
              </Table.Td>
              <Table.Td>{h.reason ?? '-'}</Table.Td>
              <Table.Td>{h.actor_display_name ?? h.actor_kind}</Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Modal>
  )
}
