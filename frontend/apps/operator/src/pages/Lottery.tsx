import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  Accordion,
  Alert,
  Badge,
  Button,
  Divider,
  Group,
  List,
  Modal,
  Paper,
  Stack,
  Table,
  Text,
  Title,
} from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'
import { useTerm } from '../term'
import { TermNotice } from '../components/TermNotice'

type Preview = components['schemas']['PreviewResponse']
type TargetPlan = components['schemas']['TargetPlan']
type Run = components['schemas']['RunResponse']

const resultLabel = (r: string) => (r === 'selected' ? '당첨' : r === 'waitlisted' ? '대기' : r)

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}
async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function Lottery() {
  const qc = useQueryClient()
  const { termId } = useTerm()
  const [preview, setPreview] = useState<Preview | null>(null)
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [resultsRun, setResultsRun] = useState<Run | null>(null)

  const offerings = useQuery({ queryKey: ['offerings', null], queryFn: () => unwrap(api.GET('/api/v1/offerings')) })
  const courses = useQuery({ queryKey: ['courses'], queryFn: () => unwrap(api.GET('/api/v1/courses')) })
  const members = useQuery({ queryKey: ['members'], queryFn: () => unwrap(api.GET('/api/v1/members')) })
  const registrations = useQuery({
    queryKey: ['registrations', null, null, null],
    queryFn: () => unwrap(api.GET('/api/v1/registrations')),
  })

  const courseName = (id: number) => courses.data?.find((c) => c.id === id)?.name ?? id
  const offeringLabel = (id: number) => {
    const o = offerings.data?.find((x) => x.id === id)
    return o ? `${courseName(o.course_id)}${o.section_label ? ` ${o.section_label}` : ''}` : String(id)
  }
  const memberByReg = (regId: number) => {
    const reg = registrations.data?.find((r) => r.id === regId)
    const m = reg && members.data?.find((x) => x.id === reg.member_id)
    return m ? `${m.name} (${m.member_no})` : `#${regId}`
  }

  const runs = useQuery({
    queryKey: ['lottery-runs', termId],
    enabled: termId !== null,
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/lottery/runs', {
          params: { query: { term_id: termId ?? undefined } },
        }),
      ),
  })

  const doPreview = useMutation({
    mutationFn: () =>
      unwrap(api.POST('/api/v1/lottery/preview', { body: { term_id: termId! } })),
    onSuccess: (data) => setPreview(data),
  })

  const doCommit = useMutation({
    mutationFn: () =>
      unwrap(
        api.POST('/api/v1/lottery/commit', {
          body: { term_id: termId!, seed: preview!.seed },
        }),
      ),
    onSuccess: () => {
      setConfirmOpen(false)
      setPreview(null)
      qc.invalidateQueries({ queryKey: ['lottery-runs'] })
      qc.invalidateQueries({ queryKey: ['registrations'] })
    },
  })

  const counts = (t: TargetPlan) => ({
    selected: t.results.filter((r) => r.result === 'selected').length,
    waitlisted: t.results.filter((r) => r.result === 'waitlisted').length,
  })

  if (!termId) return <TermNotice />

  return (
    <Stack>
      <Title order={4}>강좌 추첨</Title>

      <Group>
        <Button loading={doPreview.isPending} onClick={() => doPreview.mutate()}>
          미리보기
        </Button>
      </Group>
      {doPreview.isError && <Alert color="red">{errMessage(doPreview.error)}</Alert>}

      {preview && (
        <Paper withBorder p="md">
          <Group justify="space-between" mb="sm">
            <Group>
              <Badge color="orange" variant="light">
                미리보기 · 저장 안 됨
              </Badge>
              <Text size="sm" c="dimmed">
                seed {preview.seed}
              </Text>
            </Group>
            <Group>
              <Button variant="default" onClick={() => doPreview.mutate()} loading={doPreview.isPending}>
                다시 뽑기
              </Button>
              <Button color="teal" onClick={() => setConfirmOpen(true)} disabled={!preview.offerings.length}>
                이 결과로 확정
              </Button>
            </Group>
          </Group>

          {preview.offerings.length === 0 ? (
            <Text c="dimmed">신청자가 있는 개설 강좌가 없습니다.</Text>
          ) : (
            <Accordion variant="separated">
              {preview.offerings.map((t) => {
                const c = counts(t)
                return (
                  <Accordion.Item key={t.offering_id} value={String(t.offering_id)}>
                    <Accordion.Control>
                      <Group>
                        <Text fw={500}>{offeringLabel(t.offering_id)}</Text>
                        <Badge color="teal">당첨 {c.selected}</Badge>
                        <Badge color="yellow">대기 {c.waitlisted}</Badge>
                        <Text size="sm" c="dimmed">
                          신청 {t.eligible_count}
                          {t.capacity_type === 'gender_split' &&
                            ` (남 ${t.eligible_male} / 여 ${t.eligible_female})`}
                        </Text>
                      </Group>
                    </Accordion.Control>
                    <Accordion.Panel>
                      <ResultLists results={t.results} memberByReg={memberByReg} />
                    </Accordion.Panel>
                  </Accordion.Item>
                )
              })}
            </Accordion>
          )}
        </Paper>
      )}

      <Divider label="지난 추첨" labelPosition="left" mt="md" />
      <Table striped>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>일시</Table.Th>
            <Table.Th>seed</Table.Th>
            <Table.Th>실행자</Table.Th>
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {runs.data?.map((r) => (
            <Table.Tr key={r.id}>
              <Table.Td>{r.created_at}</Table.Td>
              <Table.Td>{r.seed}</Table.Td>
              <Table.Td>{r.executed_by_operator_id ?? '-'}</Table.Td>
              <Table.Td>
                <Button size="xs" variant="subtle" onClick={() => setResultsRun(r)}>
                  결과 보기
                </Button>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
      {runs.data?.length === 0 && (
        <Text c="dimmed" size="sm">
          이 학기의 추첨 기록이 없습니다.
        </Text>
      )}

      {/* 확정 확인 */}
      <Modal opened={confirmOpen} onClose={() => setConfirmOpen(false)} title="추첨 확정">
        <Stack>
          <Text size="sm">
            이 결과(seed {preview?.seed})로 확정합니다. 당첨/대기 상태가 수강신청에 반영됩니다.
          </Text>
          {doCommit.isError && <Alert color="red">{errMessage(doCommit.error)}</Alert>}
          <Group justify="flex-end">
            <Button variant="default" onClick={() => setConfirmOpen(false)}>
              취소
            </Button>
            <Button color="teal" loading={doCommit.isPending} onClick={() => doCommit.mutate()}>
              확정
            </Button>
          </Group>
        </Stack>
      </Modal>

      {/* 지난 추첨 결과 */}
      <RunResultsModal run={resultsRun} onClose={() => setResultsRun(null)} memberByReg={memberByReg} />
    </Stack>
  )
}

function ResultLists({
  results,
  memberByReg,
}: {
  results: components['schemas']['ResultItem'][]
  memberByReg: (id: number) => string
}) {
  const selected = results.filter((r) => r.result === 'selected').sort((a, b) => a.result_order - b.result_order)
  const waitlisted = results.filter((r) => r.result === 'waitlisted').sort((a, b) => a.result_order - b.result_order)
  return (
    <Group align="flex-start" grow>
      <div>
        <Text fw={500} size="sm" mb={4}>
          당첨
        </Text>
        <List size="sm" type="ordered">
          {selected.map((r) => (
            <List.Item key={r.registration_id}>{memberByReg(r.registration_id)}</List.Item>
          ))}
        </List>
      </div>
      <div>
        <Text fw={500} size="sm" mb={4}>
          대기
        </Text>
        <List size="sm" type="ordered">
          {waitlisted.map((r) => (
            <List.Item key={r.registration_id}>{memberByReg(r.registration_id)}</List.Item>
          ))}
        </List>
      </div>
    </Group>
  )
}

function RunResultsModal({
  run,
  onClose,
  memberByReg,
}: {
  run: Run | null
  onClose: () => void
  memberByReg: (id: number) => string
}) {
  const results = useQuery({
    queryKey: ['lottery-results', run?.id],
    enabled: run !== null,
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/lottery/runs/{run_id}/results', {
          params: { path: { run_id: run!.id } },
        }),
      ),
  })
  return (
    <Modal opened={run !== null} onClose={onClose} title="추첨 결과" size="lg">
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>회원</Table.Th>
            <Table.Th>결과</Table.Th>
            <Table.Th>순번</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {results.data?.map((r) => (
            <Table.Tr key={r.id}>
              <Table.Td>{memberByReg(r.registration_id)}</Table.Td>
              <Table.Td>
                <Badge color={r.result === 'selected' ? 'teal' : 'yellow'}>
                  {resultLabel(r.result)}
                </Badge>
              </Table.Td>
              <Table.Td>{r.result_order}</Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Modal>
  )
}
