import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Badge, Button, Group, Modal, Select, Stack, Table, Text, Title } from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'
import { useTerm } from '../term'
import { TermNotice } from '../components/TermNotice'

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

async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function Registrations() {
  const { termId } = useTerm()
  const [offering, setOffering] = useState<string | null>(null)
  const [status, setStatus] = useState<string | null>(null)
  const [historyReg, setHistoryReg] = useState<Registration | null>(null)

  const members = useQuery({ queryKey: ['members'], queryFn: () => unwrap(api.GET('/api/v1/members')) })
  const offerings = useQuery({ queryKey: ['offerings', null], queryFn: () => unwrap(api.GET('/api/v1/offerings')) })
  const courses = useQuery({ queryKey: ['courses'], queryFn: () => unwrap(api.GET('/api/v1/courses')) })

  const courseName = (id: number) => courses.data?.find((c) => c.id === id)?.name ?? id
  const memberName = (id: number) => {
    const m = members.data?.find((x) => x.id === id)
    return m ? `${m.name} (${m.member_no})` : String(id)
  }
  const offeringLabel = (id: number) => {
    const o = offerings.data?.find((x) => x.id === id)
    return o ? `${courseName(o.course_id)}${o.section_label ? ` ${o.section_label}` : ''}` : String(id)
  }

  const list = useQuery({
    queryKey: ['registrations', 'overview', termId, offering, status],
    enabled: termId !== null,
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/registrations', {
          params: {
            query: {
              term_id: termId ?? undefined,
              offering_id: offering ? Number(offering) : undefined,
              status: status || undefined,
            },
          },
        }),
      ),
  })

  const offeringOptions = (offerings.data ?? [])
    .filter((o) => o.term_id === termId)
    .map((o) => ({ value: String(o.id), label: offeringLabel(o.id) }))

  if (!termId) return <TermNotice />

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={4}>신청 현황</Title>
        <Group>
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
        </Group>
      </Group>

      <Text size="sm" c="dimmed">
        조회 전용입니다. 신청·취소는 &lsquo;수강접수&rsquo;에서 하세요.
      </Text>

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
                  <Button size="xs" variant="subtle" onClick={() => setHistoryReg(r)}>
                    이력
                  </Button>
                </Table.Td>
              </Table.Tr>
            )
          })}
        </Table.Tbody>
      </Table>

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
