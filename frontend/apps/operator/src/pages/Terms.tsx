import { Badge, NumberInput, Select, TextInput, Title } from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'
import { CrudMaster } from '../components/CrudMaster'

type Term = components['schemas']['TermResponse']

const TERM_STATUS = [
  { value: 'draft', label: '준비' },
  { value: 'open', label: '신청중' },
  { value: 'closed', label: '마감' },
  { value: 'finalized', label: '확정' },
]
const termStatusLabel = (s: string) => TERM_STATUS.find((x) => x.value === s)?.label ?? s

async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function Terms() {
  return (
    <>
      <Title order={4} mb="md">
        학기 관리
      </Title>
      <CrudMaster<Term, { name: string; status: Term['status']; max_registrations_per_member: number; starts_on: string; ends_on: string }>
        addLabel="학기 추가"
        queryKey={['terms']}
        fetchList={() => unwrap(api.GET('/api/v1/terms'))}
        onDelete={(r) =>
          unwrap(api.DELETE('/api/v1/terms/{term_id}', { params: { path: { term_id: r.id } } })).then(() => undefined)
        }
        rowLabel={(r) => r.name}
        create={(v) =>
          unwrap(
            api.POST('/api/v1/terms', {
              body: {
                name: v.name,
                status: v.status,
                max_registrations_per_member: v.max_registrations_per_member,
                starts_on: v.starts_on || null,
                ends_on: v.ends_on || null,
              },
            }),
          ).then(() => undefined)
        }
        update={(id, v) =>
          unwrap(
            api.PATCH('/api/v1/terms/{term_id}', {
              params: { path: { term_id: id } },
              body: {
                name: v.name,
                status: v.status,
                max_registrations_per_member: v.max_registrations_per_member,
                starts_on: v.starts_on || null,
                ends_on: v.ends_on || null,
              },
            }),
          ).then(() => undefined)
        }
        columns={[
          { head: '이름', cell: (r) => r.name },
          { head: '상태', cell: (r) => <Badge>{termStatusLabel(r.status)}</Badge> },
          { head: '인당 최대', cell: (r) => r.max_registrations_per_member || '무제한' },
          { head: '기간', cell: (r) => [r.starts_on, r.ends_on].filter(Boolean).join(' ~ ') || '-' },
        ]}
        initial={{ name: '', status: 'draft', max_registrations_per_member: 0, starts_on: '', ends_on: '' }}
        toValues={(r) => ({
          name: r.name,
          status: r.status,
          max_registrations_per_member: r.max_registrations_per_member,
          starts_on: r.starts_on ?? '',
          ends_on: r.ends_on ?? '',
        })}
        validate={{ name: (v: string) => (v.trim() ? null : '이름을 입력하세요.') }}
        renderForm={(form) => (
          <>
            <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
            <Select label="상태" data={TERM_STATUS} allowDeselect={false} {...form.getInputProps('status')} />
            <NumberInput
              label="인당 최대 신청 수 (0=무제한)"
              min={0}
              {...form.getInputProps('max_registrations_per_member')}
            />
            <TextInput type="date" label="시작일" {...form.getInputProps('starts_on')} />
            <TextInput type="date" label="종료일" {...form.getInputProps('ends_on')} />
          </>
        )}
      />
    </>
  )
}
