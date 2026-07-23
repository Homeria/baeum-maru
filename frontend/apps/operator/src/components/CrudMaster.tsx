import { useRef, useState, type ReactNode } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useDisclosure } from '@mantine/hooks'
import { useForm, type UseFormReturnType } from '@mantine/form'
import { Alert, Button, Group, Modal, Stack, Table, Text } from '@mantine/core'

// 여러 기준정보 탭이 공유하는 목록·생성·수정·삭제 배관.
// API 호출은 각 엔티티가 타입 있는 클로저로 주입 → 타입 안전성은 호출부에 남는다.
export type CrudConfig<Row extends { id: number }, Values extends Record<string, unknown>> = {
  addLabel: string
  queryKey: unknown[]
  fetchList: () => Promise<Row[]>
  create: (values: Values) => Promise<void>
  update: (id: number, values: Values) => Promise<void>
  columns: { head: string; cell: (row: Row) => ReactNode }[]
  initial: Values
  toValues: (row: Row) => Values
  renderForm: (form: UseFormReturnType<Values>) => ReactNode
  validate?: Record<string, (value: never) => string | null>
  // 수정 버튼 앞에 붙일 추가 행 액션(예: 하위 항목 관리)
  rowActions?: (row: Row) => ReactNode
  // 있으면 삭제 버튼 노출. 참조 중이면 백엔드가 409를 주고 그 메시지를 안내한다.
  onDelete?: (row: Row) => Promise<void>
  rowLabel?: (row: Row) => string
}

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}

export function CrudMaster<Row extends { id: number }, Values extends Record<string, unknown>>(
  config: CrudConfig<Row, Values>,
) {
  const qc = useQueryClient()
  const [opened, modal] = useDisclosure(false)
  const [deleting, setDeleting] = useState<Row | null>(null)
  const list = useQuery({ queryKey: config.queryKey, queryFn: config.fetchList })
  const form = useForm<Values>({
    initialValues: config.initial,
    validate: config.validate as never,
  })
  const editingId = useRef<number | null>(null)

  const openCreate = () => {
    editingId.current = null
    form.setValues(config.initial)
    modal.open()
  }
  const openEdit = (row: Row) => {
    editingId.current = row.id
    form.setValues(config.toValues(row))
    modal.open()
  }

  const save = useMutation({
    mutationFn: async (values: Values) => {
      if (editingId.current === null) await config.create(values)
      else await config.update(editingId.current, values)
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: config.queryKey })
      modal.close()
    },
  })

  const remove = useMutation({
    mutationFn: async (row: Row) => {
      await config.onDelete!(row)
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: config.queryKey })
      setDeleting(null)
    },
  })

  return (
    <Stack>
      <Group justify="flex-end">
        <Button onClick={openCreate}>{config.addLabel}</Button>
      </Group>

      {list.isError && <Alert color="red">{errMessage(list.error)}</Alert>}

      <Table striped highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            {config.columns.map((c) => (
              <Table.Th key={c.head}>{c.head}</Table.Th>
            ))}
            <Table.Th />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {list.data?.map((row) => (
            <Table.Tr key={row.id}>
              {config.columns.map((c) => (
                <Table.Td key={c.head}>{c.cell(row)}</Table.Td>
              ))}
              <Table.Td>
                <Group gap="xs" justify="flex-end">
                  {config.rowActions?.(row)}
                  <Button size="xs" variant="subtle" onClick={() => openEdit(row)}>
                    수정
                  </Button>
                  {config.onDelete && (
                    <Button size="xs" variant="subtle" color="red" onClick={() => setDeleting(row)}>
                      삭제
                    </Button>
                  )}
                </Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>

      <Modal opened={opened} onClose={modal.close} title={config.addLabel}>
        <form onSubmit={form.onSubmit((v) => save.mutate(v))}>
          <Stack>
            {config.renderForm(form)}
            {save.isError && <Alert color="red">{errMessage(save.error)}</Alert>}
            <Group justify="flex-end">
              <Button variant="default" onClick={modal.close}>
                취소
              </Button>
              <Button type="submit" loading={save.isPending}>
                저장
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>

      <Modal opened={deleting !== null} onClose={() => setDeleting(null)} title="삭제 확인">
        <Stack>
          <Text size="sm">
            {deleting && config.rowLabel ? config.rowLabel(deleting) : '이 항목'}을(를) 삭제할까요?
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
