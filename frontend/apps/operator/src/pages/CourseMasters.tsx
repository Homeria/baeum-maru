import { useQuery } from '@tanstack/react-query'
import { Badge, NumberInput, Select, Switch, Tabs, TextInput, Title } from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'
import { CrudMaster } from '../components/CrudMaster'

type Category = components['schemas']['CourseCategoryResponse']
type Level = components['schemas']['CourseLevelResponse']
type Instructor = components['schemas']['InstructorResponse']
type Term = components['schemas']['TermResponse']
type TimeSlot = components['schemas']['TimeSlotResponse']
type Course = components['schemas']['CourseResponse']

const TERM_STATUS = [
  { value: 'draft', label: '준비' },
  { value: 'open', label: '신청중' },
  { value: 'closed', label: '마감' },
  { value: 'finalized', label: '확정' },
]
const termStatusLabel = (s: string) => TERM_STATUS.find((x) => x.value === s)?.label ?? s
const activeBadge = (active: boolean) => (
  <Badge color={active ? 'teal' : 'gray'}>{active ? '활성' : '비활성'}</Badge>
)

async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function CourseMasters() {
  // 강좌 탭의 Select 옵션에 쓸 분류·난도 목록
  const categories = useQuery({
    queryKey: ['course-categories'],
    queryFn: () => unwrap(api.GET('/api/v1/course-categories')),
  })
  const levels = useQuery({
    queryKey: ['course-levels'],
    queryFn: () => unwrap(api.GET('/api/v1/course-levels')),
  })

  return (
    <>
      <Title order={4} mb="md">
        강좌 기준정보
      </Title>
      <Tabs defaultValue="courses">
        <Tabs.List mb="md">
          <Tabs.Tab value="courses">강좌</Tabs.Tab>
          <Tabs.Tab value="categories">분류</Tabs.Tab>
          <Tabs.Tab value="levels">난도</Tabs.Tab>
          <Tabs.Tab value="instructors">강사</Tabs.Tab>
          <Tabs.Tab value="terms">학기</Tabs.Tab>
          <Tabs.Tab value="timeslots">교시</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="courses">
          <CrudMaster<Course, { category_id: string; level_id: string | null; name: string; description: string; is_active: boolean }>
            addLabel="강좌 추가"
            queryKey={['courses']}
            fetchList={() => unwrap(api.GET('/api/v1/courses'))}
            create={(v) =>
              unwrap(
                api.POST('/api/v1/courses', {
                  body: {
                    category_id: Number(v.category_id),
                    level_id: v.level_id ? Number(v.level_id) : null,
                    name: v.name,
                    description: v.description || null,
                  },
                }),
              ).then(() => undefined)
            }
            update={(id, v) =>
              unwrap(
                api.PATCH('/api/v1/courses/{course_id}', {
                  params: { path: { course_id: id } },
                  body: {
                    category_id: Number(v.category_id),
                    level_id: v.level_id ? Number(v.level_id) : null,
                    name: v.name,
                    description: v.description || null,
                    is_active: v.is_active,
                  },
                }),
              ).then(() => undefined)
            }
            columns={[
              { head: '이름', cell: (r) => r.name },
              {
                head: '분류',
                cell: (r) => categories.data?.find((c) => c.id === r.category_id)?.name ?? r.category_id,
              },
              {
                head: '난도',
                cell: (r) => levels.data?.find((l) => l.id === r.level_id)?.name ?? '-',
              },
              { head: '상태', cell: (r) => activeBadge(r.is_active) },
            ]}
            initial={{ category_id: '', level_id: null, name: '', description: '', is_active: true }}
            toValues={(r) => ({
              category_id: String(r.category_id),
              level_id: r.level_id ? String(r.level_id) : null,
              name: r.name,
              description: r.description ?? '',
              is_active: r.is_active,
            })}
            validate={{
              category_id: (v: string) => (v ? null : '분류를 선택하세요.'),
              name: (v: string) => (v.trim() ? null : '이름을 입력하세요.'),
            }}
            renderForm={(form) => (
              <>
                <Select
                  label="분류"
                  withAsterisk
                  data={(categories.data ?? []).map((c) => ({ value: String(c.id), label: c.name }))}
                  {...form.getInputProps('category_id')}
                />
                <Select
                  label="난도"
                  clearable
                  data={(levels.data ?? []).map((l) => ({ value: String(l.id), label: l.name }))}
                  {...form.getInputProps('level_id')}
                />
                <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
                <TextInput label="설명" {...form.getInputProps('description')} />
                <Switch label="활성" {...form.getInputProps('is_active', { type: 'checkbox' })} />
              </>
            )}
          />
        </Tabs.Panel>

        <Tabs.Panel value="categories">
          <CrudMaster<Category, { name: string; sort_order: number; is_active: boolean }>
            addLabel="분류 추가"
            queryKey={['course-categories']}
            fetchList={() => unwrap(api.GET('/api/v1/course-categories'))}
            create={(v) =>
              unwrap(api.POST('/api/v1/course-categories', { body: { name: v.name, sort_order: v.sort_order } })).then(() => undefined)
            }
            update={(id, v) =>
              unwrap(
                api.PATCH('/api/v1/course-categories/{category_id}', {
                  params: { path: { category_id: id } },
                  body: v,
                }),
              ).then(() => undefined)
            }
            columns={[
              { head: '이름', cell: (r) => r.name },
              { head: '정렬', cell: (r) => r.sort_order },
              { head: '상태', cell: (r) => activeBadge(r.is_active) },
            ]}
            initial={{ name: '', sort_order: 0, is_active: true }}
            toValues={(r) => ({ name: r.name, sort_order: r.sort_order, is_active: r.is_active })}
            validate={{ name: (v: string) => (v.trim() ? null : '이름을 입력하세요.') }}
            renderForm={(form) => (
              <>
                <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
                <NumberInput label="정렬 순서" {...form.getInputProps('sort_order')} />
                <Switch label="활성" {...form.getInputProps('is_active', { type: 'checkbox' })} />
              </>
            )}
          />
        </Tabs.Panel>

        <Tabs.Panel value="levels">
          <CrudMaster<Level, { name: string; sort_order: number; is_active: boolean }>
            addLabel="난도 추가"
            queryKey={['course-levels']}
            fetchList={() => unwrap(api.GET('/api/v1/course-levels'))}
            create={(v) =>
              unwrap(api.POST('/api/v1/course-levels', { body: { name: v.name, sort_order: v.sort_order } })).then(() => undefined)
            }
            update={(id, v) =>
              unwrap(
                api.PATCH('/api/v1/course-levels/{level_id}', {
                  params: { path: { level_id: id } },
                  body: v,
                }),
              ).then(() => undefined)
            }
            columns={[
              { head: '이름', cell: (r) => r.name },
              { head: '정렬', cell: (r) => r.sort_order },
              { head: '상태', cell: (r) => activeBadge(r.is_active) },
            ]}
            initial={{ name: '', sort_order: 0, is_active: true }}
            toValues={(r) => ({ name: r.name, sort_order: r.sort_order, is_active: r.is_active })}
            validate={{ name: (v: string) => (v.trim() ? null : '이름을 입력하세요.') }}
            renderForm={(form) => (
              <>
                <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
                <NumberInput label="정렬 순서" {...form.getInputProps('sort_order')} />
                <Switch label="활성" {...form.getInputProps('is_active', { type: 'checkbox' })} />
              </>
            )}
          />
        </Tabs.Panel>

        <Tabs.Panel value="instructors">
          <CrudMaster<Instructor, { name: string; phone: string; is_active: boolean }>
            addLabel="강사 추가"
            queryKey={['instructors']}
            fetchList={() => unwrap(api.GET('/api/v1/instructors'))}
            create={(v) =>
              unwrap(api.POST('/api/v1/instructors', { body: { name: v.name, phone: v.phone || null } })).then(() => undefined)
            }
            update={(id, v) =>
              unwrap(
                api.PATCH('/api/v1/instructors/{instructor_id}', {
                  params: { path: { instructor_id: id } },
                  body: { name: v.name, phone: v.phone || null, is_active: v.is_active },
                }),
              ).then(() => undefined)
            }
            columns={[
              { head: '이름', cell: (r) => r.name },
              { head: '연락처', cell: (r) => r.phone ?? '-' },
              { head: '상태', cell: (r) => activeBadge(r.is_active) },
            ]}
            initial={{ name: '', phone: '', is_active: true }}
            toValues={(r) => ({ name: r.name, phone: r.phone ?? '', is_active: r.is_active })}
            validate={{ name: (v: string) => (v.trim() ? null : '이름을 입력하세요.') }}
            renderForm={(form) => (
              <>
                <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
                <TextInput label="연락처" {...form.getInputProps('phone')} />
                <Switch label="활성" {...form.getInputProps('is_active', { type: 'checkbox' })} />
              </>
            )}
          />
        </Tabs.Panel>

        <Tabs.Panel value="terms">
          <CrudMaster<Term, { name: string; status: Term['status']; max_registrations_per_member: number; starts_on: string; ends_on: string }>
            addLabel="학기 추가"
            queryKey={['terms']}
            fetchList={() => unwrap(api.GET('/api/v1/terms'))}
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
              {
                head: '인당 최대',
                cell: (r) => (r.max_registrations_per_member || '무제한'),
              },
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
        </Tabs.Panel>

        <Tabs.Panel value="timeslots">
          <CrudMaster<TimeSlot, { name: string; start_time: string; end_time: string; sort_order: number; is_active: boolean }>
            addLabel="교시 추가"
            queryKey={['time-slots']}
            fetchList={() => unwrap(api.GET('/api/v1/time-slots'))}
            create={(v) =>
              unwrap(
                api.POST('/api/v1/time-slots', {
                  body: { name: v.name, start_time: v.start_time, end_time: v.end_time, sort_order: v.sort_order },
                }),
              ).then(() => undefined)
            }
            update={(id, v) =>
              unwrap(
                api.PATCH('/api/v1/time-slots/{time_slot_id}', {
                  params: { path: { time_slot_id: id } },
                  body: v,
                }),
              ).then(() => undefined)
            }
            columns={[
              { head: '이름', cell: (r) => r.name },
              { head: '시작', cell: (r) => r.start_time },
              { head: '종료', cell: (r) => r.end_time },
              { head: '상태', cell: (r) => activeBadge(r.is_active) },
            ]}
            initial={{ name: '', start_time: '09:00', end_time: '10:00', sort_order: 0, is_active: true }}
            toValues={(r) => ({
              name: r.name,
              start_time: r.start_time,
              end_time: r.end_time,
              sort_order: r.sort_order,
              is_active: r.is_active,
            })}
            validate={{ name: (v: string) => (v.trim() ? null : '이름을 입력하세요.') }}
            renderForm={(form) => (
              <>
                <TextInput label="이름" withAsterisk placeholder="예: 1교시" {...form.getInputProps('name')} />
                <TextInput type="time" label="시작" {...form.getInputProps('start_time')} />
                <TextInput type="time" label="종료" {...form.getInputProps('end_time')} />
                <NumberInput label="정렬 순서" {...form.getInputProps('sort_order')} />
                <Switch label="활성" {...form.getInputProps('is_active', { type: 'checkbox' })} />
              </>
            )}
          />
        </Tabs.Panel>
      </Tabs>
    </>
  )
}
