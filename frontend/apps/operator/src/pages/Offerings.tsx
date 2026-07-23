import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useDisclosure } from '@mantine/hooks'
import { useForm } from '@mantine/form'
import { useNavigate } from 'react-router-dom'
import {
  Alert,
  Badge,
  Button,
  Group,
  Modal,
  NumberInput,
  Select,
  Stack,
  Table,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'

type Offering = components['schemas']['OfferingResponse']
type Schedule = components['schemas']['ScheduleResponse']

const CAPACITY = [
  { value: 'fixed', label: '정원제' },
  { value: 'open', label: '제한없음' },
  { value: 'gender_split', label: '남녀분리' },
]
const STATUS = [
  { value: 'draft', label: '준비' },
  { value: 'open', label: '신청중' },
  { value: 'closed', label: '마감' },
  { value: 'cancelled', label: '취소' },
]
const WEEKDAYS = ['월', '화', '수', '목', '금', '토', '일']
const label = (opts: { value: string; label: string }[], v: string) =>
  opts.find((o) => o.value === v)?.label ?? v

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}
async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

type OfferingValues = {
  term_id: string
  course_id: string
  section_label: string
  instructor_id: string | null
  capacity_type: Offering['capacity_type']
  capacity_total: number | string
  male_capacity: number | string
  female_capacity: number | string
  status: Offering['status']
  sort_order: number
}

const EMPTY: OfferingValues = {
  term_id: '',
  course_id: '',
  section_label: '',
  instructor_id: null,
  capacity_type: 'fixed',
  capacity_total: '',
  male_capacity: '',
  female_capacity: '',
  status: 'draft',
  sort_order: 0,
}

const num = (x: number | string) => (x === '' || x == null ? null : Number(x))

function buildBody(v: OfferingValues) {
  return {
    term_id: Number(v.term_id),
    course_id: Number(v.course_id),
    section_label: v.section_label || null,
    instructor_id: v.instructor_id ? Number(v.instructor_id) : null,
    capacity_type: v.capacity_type,
    capacity_total: v.capacity_type === 'fixed' ? num(v.capacity_total) : null,
    male_capacity: v.capacity_type === 'gender_split' ? num(v.male_capacity) : null,
    female_capacity: v.capacity_type === 'gender_split' ? num(v.female_capacity) : null,
    status: v.status,
    sort_order: v.sort_order,
  }
}

export function Offerings() {
  const qc = useQueryClient()
  const navigate = useNavigate()
  const [termFilter, setTermFilter] = useState<string | null>(null)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [formOpen, form] = useDisclosure(false)
  const [schedOffering, setSchedOffering] = useState<Offering | null>(null)
  const [deleting, setDeleting] = useState<Offering | null>(null)

  const terms = useQuery({ queryKey: ['terms'], queryFn: () => unwrap(api.GET('/api/v1/terms')) })
  const courses = useQuery({ queryKey: ['courses'], queryFn: () => unwrap(api.GET('/api/v1/courses')) })
  const instructors = useQuery({
    queryKey: ['instructors'],
    queryFn: () => unwrap(api.GET('/api/v1/instructors')),
  })

  const list = useQuery({
    queryKey: ['offerings', termFilter],
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/offerings', {
          params: { query: { term_id: termFilter ? Number(termFilter) : undefined } },
        }),
      ),
  })

  const formHook = useForm<OfferingValues>({
    initialValues: EMPTY,
    validate: {
      term_id: (v) => (v ? null : '학기를 선택하세요.'),
      course_id: (v) => (v ? null : '강좌를 선택하세요.'),
    },
  })

  const openCreate = () => {
    setEditingId(null)
    formHook.setValues(EMPTY)
    form.open()
  }
  const openEdit = (o: Offering) => {
    setEditingId(o.id)
    formHook.setValues({
      term_id: String(o.term_id),
      course_id: String(o.course_id),
      section_label: o.section_label ?? '',
      instructor_id: o.instructor_id ? String(o.instructor_id) : null,
      capacity_type: o.capacity_type,
      capacity_total: o.capacity_total ?? '',
      male_capacity: o.male_capacity ?? '',
      female_capacity: o.female_capacity ?? '',
      status: o.status,
      sort_order: o.sort_order,
    })
    form.open()
  }

  const save = useMutation({
    mutationFn: async (v: OfferingValues) => {
      if (editingId === null) {
        await unwrap(api.POST('/api/v1/offerings', { body: buildBody(v) }))
      } else {
        await unwrap(
          api.PATCH('/api/v1/offerings/{offering_id}', {
            params: { path: { offering_id: editingId } },
            body: buildBody(v),
          }),
        )
      }
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['offerings'] })
      form.close()
    },
  })

  const remove = useMutation({
    mutationFn: async (o: Offering) => {
      await unwrap(
        api.DELETE('/api/v1/offerings/{offering_id}', { params: { path: { offering_id: o.id } } }),
      )
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['offerings'] })
      setDeleting(null)
    },
  })

  const courseName = (id: number) => courses.data?.find((c) => c.id === id)?.name ?? id
  const termName = (id: number) => terms.data?.find((t) => t.id === id)?.name ?? id
  const instructorName = (id: number | null) =>
    id ? (instructors.data?.find((i) => i.id === id)?.name ?? id) : '-'
  const capacityText = (o: Offering) =>
    o.capacity_type === 'fixed'
      ? String(o.capacity_total ?? '-')
      : o.capacity_type === 'gender_split'
        ? `남 ${o.male_capacity ?? '-'} / 여 ${o.female_capacity ?? '-'}`
        : '제한없음'

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={4}>개설 강좌</Title>
        <Group>
          <Select
            placeholder="학기 전체"
            clearable
            data={(terms.data ?? []).map((t) => ({ value: String(t.id), label: t.name }))}
            value={termFilter}
            onChange={setTermFilter}
            w={200}
          />
          <Button onClick={openCreate}>개설 추가</Button>
        </Group>
      </Group>

      {list.isError && <Alert color="red">{errMessage(list.error)}</Alert>}

      <Table striped highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>강좌</Table.Th>
            <Table.Th>학기</Table.Th>
            <Table.Th>분반</Table.Th>
            <Table.Th>강사</Table.Th>
            <Table.Th>정원</Table.Th>
            <Table.Th>상태</Table.Th>
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {list.data?.map((o) => (
            <Table.Tr key={o.id}>
              <Table.Td>{courseName(o.course_id)}</Table.Td>
              <Table.Td>{termName(o.term_id)}</Table.Td>
              <Table.Td>{o.section_label ?? '-'}</Table.Td>
              <Table.Td>{instructorName(o.instructor_id)}</Table.Td>
              <Table.Td>{capacityText(o)}</Table.Td>
              <Table.Td>
                <Badge>{label(STATUS, o.status)}</Badge>
              </Table.Td>
              <Table.Td>
                <Group gap="xs" justify="flex-end">
                  <Button size="xs" variant="light" onClick={() => setSchedOffering(o)}>
                    시간표
                  </Button>
                  <Button size="xs" variant="subtle" onClick={() => openEdit(o)}>
                    수정
                  </Button>
                  <Button size="xs" variant="subtle" color="red" onClick={() => setDeleting(o)}>
                    삭제
                  </Button>
                </Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>

      <Modal opened={formOpen} onClose={form.close} title={editingId ? '개설 수정' : '개설 추가'}>
        <form onSubmit={formHook.onSubmit((v) => save.mutate(v))}>
          <Stack>
            {terms.data?.length === 0 && (
              <Alert color="yellow">
                등록된 학기가 없습니다.{' '}
                <Button variant="subtle" size="compact-xs" onClick={() => navigate('/course-masters?tab=terms')}>
                  강좌 기준정보 › 학기로 이동
                </Button>
              </Alert>
            )}
            {courses.data?.length === 0 && (
              <Alert color="yellow">
                등록된 강좌가 없습니다.{' '}
                <Button variant="subtle" size="compact-xs" onClick={() => navigate('/course-masters?tab=courses')}>
                  강좌 기준정보 › 강좌로 이동
                </Button>
              </Alert>
            )}
            <Select
              label="학기"
              withAsterisk
              data={(terms.data ?? []).map((t) => ({ value: String(t.id), label: t.name }))}
              {...formHook.getInputProps('term_id')}
            />
            <Select
              label="강좌"
              withAsterisk
              searchable
              data={(courses.data ?? []).map((c) => ({ value: String(c.id), label: c.name }))}
              {...formHook.getInputProps('course_id')}
            />
            <TextInput label="분반" placeholder="예: A반" {...formHook.getInputProps('section_label')} />
            <Select
              label="강사"
              clearable
              searchable
              data={(instructors.data ?? []).map((i) => ({ value: String(i.id), label: i.name }))}
              {...formHook.getInputProps('instructor_id')}
            />
            <Select
              label="정원 유형"
              allowDeselect={false}
              data={CAPACITY}
              {...formHook.getInputProps('capacity_type')}
            />
            {formHook.values.capacity_type === 'fixed' && (
              <NumberInput label="정원" min={0} {...formHook.getInputProps('capacity_total')} />
            )}
            {formHook.values.capacity_type === 'gender_split' && (
              <Group grow>
                <NumberInput label="남 정원" min={0} {...formHook.getInputProps('male_capacity')} />
                <NumberInput label="여 정원" min={0} {...formHook.getInputProps('female_capacity')} />
              </Group>
            )}
            <Select
              label="상태"
              allowDeselect={false}
              data={STATUS}
              {...formHook.getInputProps('status')}
            />
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

      <Modal opened={deleting !== null} onClose={() => setDeleting(null)} title="개설 삭제">
        <Stack>
          <Text size="sm">
            {deleting && `${courseName(deleting.course_id)}${deleting.section_label ? ` ${deleting.section_label}` : ''}`} 개설을 삭제할까요?
          </Text>
          <Text size="xs" c="dimmed">
            신청·시간표가 있으면 삭제되지 않습니다. 그때는 상태를 '취소'로 두세요.
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

      <SchedulesModal offering={schedOffering} onClose={() => setSchedOffering(null)} />
    </Stack>
  )
}

function SchedulesModal({ offering, onClose }: { offering: Offering | null; onClose: () => void }) {
  const qc = useQueryClient()
  const navigate = useNavigate()
  const timeslots = useQuery({
    queryKey: ['time-slots'],
    queryFn: () => unwrap(api.GET('/api/v1/time-slots')),
  })
  const spaces = useQuery({ queryKey: ['spaces'], queryFn: () => unwrap(api.GET('/api/v1/spaces')) })

  const schedules = useQuery({
    queryKey: ['schedules', offering?.id],
    enabled: offering !== null,
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/offerings/{offering_id}/schedules', {
          params: { path: { offering_id: offering!.id } },
        }),
      ),
  })

  const form = useForm({ initialValues: { weekday: '1', time_slot_id: '', space_id: '' } })

  const add = useMutation({
    mutationFn: async (v: typeof form.values) => {
      await unwrap(
        api.POST('/api/v1/offerings/{offering_id}/schedules', {
          params: { path: { offering_id: offering!.id } },
          body: {
            weekday: Number(v.weekday),
            time_slot_id: Number(v.time_slot_id),
            space_id: Number(v.space_id),
          },
        }),
      )
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['schedules', offering?.id] })
      form.reset()
    },
  })

  const remove = useMutation({
    mutationFn: async (s: Schedule) => {
      await unwrap(
        api.DELETE('/api/v1/course-schedules/{schedule_id}', {
          params: { path: { schedule_id: s.id } },
        }),
      )
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['schedules', offering?.id] }),
  })

  const tsName = (id: number) => timeslots.data?.find((t) => t.id === id)?.name ?? id
  const spName = (id: number) => spaces.data?.find((s) => s.id === id)?.name ?? id
  const noSpaces = spaces.data?.length === 0

  return (
    <Modal opened={offering !== null} onClose={onClose} title="시간표" size="lg">
      <Stack>
        <Table>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>요일</Table.Th>
              <Table.Th>교시</Table.Th>
              <Table.Th>공간</Table.Th>
              <Table.Th />
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {schedules.data?.map((s) => (
              <Table.Tr key={s.id}>
                <Table.Td>{WEEKDAYS[s.weekday - 1]}</Table.Td>
                <Table.Td>{tsName(s.time_slot_id)}</Table.Td>
                <Table.Td>{spName(s.space_id)}</Table.Td>
                <Table.Td>
                  <Button
                    size="xs"
                    color="red"
                    variant="light"
                    loading={remove.isPending && remove.variables?.id === s.id}
                    onClick={() => remove.mutate(s)}
                  >
                    삭제
                  </Button>
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
        {schedules.data?.length === 0 && (
          <Text c="dimmed" ta="center">
            등록된 시간표가 없습니다.
          </Text>
        )}

        {noSpaces ? (
          <Alert color="yellow">
            등록된 공간이 없습니다. 공간을 먼저 등록하세요.{' '}
            <Button variant="subtle" size="compact-xs" onClick={() => navigate('/spaces?tab=spaces')}>
              공간으로 이동
            </Button>
          </Alert>
        ) : (
          <form onSubmit={form.onSubmit((v) => add.mutate(v))}>
            <Group align="flex-end">
              <Select
                label="요일"
                w={90}
                allowDeselect={false}
                data={WEEKDAYS.map((w, i) => ({ value: String(i + 1), label: w }))}
                {...form.getInputProps('weekday')}
              />
              <Select
                label="교시"
                flex={1}
                data={(timeslots.data ?? []).map((t) => ({ value: String(t.id), label: t.name }))}
                {...form.getInputProps('time_slot_id')}
              />
              <Select
                label="공간"
                flex={1}
                data={(spaces.data ?? []).map((s) => ({ value: String(s.id), label: s.name }))}
                {...form.getInputProps('space_id')}
              />
              <Button
                type="submit"
                loading={add.isPending}
                disabled={!form.values.time_slot_id || !form.values.space_id}
              >
                추가
              </Button>
            </Group>
            {add.isError && (
              <Alert color="red" mt="xs">
                {errMessage(add.error)}
              </Alert>
            )}
          </form>
        )}
      </Stack>
    </Modal>
  )
}
