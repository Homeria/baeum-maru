import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useForm } from '@mantine/form'
import {
  Alert,
  Badge,
  Button,
  Group,
  Modal,
  NumberInput,
  Select,
  Stack,
  Switch,
  Table,
  Tabs,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { api } from '../api/client'
import type { components } from '../api/schema'
import { CrudMaster } from '../components/CrudMaster'

type Building = components['schemas']['BuildingResponse']
type Floor = components['schemas']['BuildingFloorResponse']
type SpaceType = components['schemas']['SpaceTypeResponse']
type Space = components['schemas']['SpaceResponse']
type FloorWithBuilding = Floor & { buildingName: string }

const activeBadge = (active: boolean) => (
  <Badge color={active ? 'teal' : 'gray'}>{active ? '활성' : '비활성'}</Badge>
)

function errMessage(error: unknown): string {
  const e = error as { error?: { message?: string } } | undefined
  return e?.error?.message ?? '요청을 처리하지 못했습니다.'
}
async function unwrap<T>(p: Promise<{ data?: T; error?: unknown }>): Promise<T> {
  const { data, error } = await p
  if (error) throw error
  return data as T
}

export function Spaces() {
  // 모든 건물의 층을 모아 공간 탭의 위치 Select·표시에 쓴다(전역 층 목록 API가 없어 합침).
  const floors = useQuery({
    queryKey: ['all-floors'],
    queryFn: async (): Promise<FloorWithBuilding[]> => {
      const buildings = await unwrap(api.GET('/api/v1/buildings'))
      const perBuilding = await Promise.all(
        buildings.map((b) =>
          unwrap(
            api.GET('/api/v1/buildings/{building_id}/floors', {
              params: { path: { building_id: b.id } },
            }),
          ).then((fs) => fs.map((f) => ({ ...f, buildingName: b.name }))),
        ),
      )
      return perBuilding.flat()
    },
  })
  const spaceTypes = useQuery({
    queryKey: ['space-types'],
    queryFn: () => unwrap(api.GET('/api/v1/space-types')),
  })

  const floorLabel = (id: number) => {
    const f = floors.data?.find((x) => x.id === id)
    return f ? `${f.buildingName} - ${f.label}` : String(id)
  }
  const typeName = (id: number) => spaceTypes.data?.find((t) => t.id === id)?.name ?? id

  return (
    <>
      <Title order={4} mb="md">
        공간
      </Title>
      <Tabs defaultValue="spaces">
        <Tabs.List mb="md">
          <Tabs.Tab value="spaces">공간</Tabs.Tab>
          <Tabs.Tab value="buildings">건물·층</Tabs.Tab>
          <Tabs.Tab value="types">공간 유형</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="spaces">
          <CrudMaster<Space, { building_floor_id: string; space_type_id: string; name: string; sort_order: number; is_active: boolean }>
            addLabel="공간 추가"
            queryKey={['spaces']}
            fetchList={() => unwrap(api.GET('/api/v1/spaces'))}
            create={(v) =>
              unwrap(
                api.POST('/api/v1/spaces', {
                  body: {
                    building_floor_id: Number(v.building_floor_id),
                    space_type_id: Number(v.space_type_id),
                    name: v.name,
                    sort_order: v.sort_order,
                  },
                }),
              ).then(() => undefined)
            }
            update={(id, v) =>
              unwrap(
                api.PATCH('/api/v1/spaces/{space_id}', {
                  params: { path: { space_id: id } },
                  body: {
                    building_floor_id: Number(v.building_floor_id),
                    space_type_id: Number(v.space_type_id),
                    name: v.name,
                    sort_order: v.sort_order,
                    is_active: v.is_active,
                  },
                }),
              ).then(() => undefined)
            }
            columns={[
              { head: '이름', cell: (r) => r.name },
              { head: '위치', cell: (r) => floorLabel(r.building_floor_id) },
              { head: '유형', cell: (r) => typeName(r.space_type_id) },
              { head: '상태', cell: (r) => activeBadge(r.is_active) },
            ]}
            initial={{ building_floor_id: '', space_type_id: '', name: '', sort_order: 0, is_active: true }}
            toValues={(r) => ({
              building_floor_id: String(r.building_floor_id),
              space_type_id: String(r.space_type_id),
              name: r.name,
              sort_order: r.sort_order,
              is_active: r.is_active,
            })}
            validate={{
              building_floor_id: (v: string) => (v ? null : '위치를 선택하세요.'),
              space_type_id: (v: string) => (v ? null : '유형을 선택하세요.'),
              name: (v: string) => (v.trim() ? null : '이름을 입력하세요.'),
            }}
            renderForm={(form) => (
              <>
                <Select
                  label="위치(건물-층)"
                  withAsterisk
                  searchable
                  data={(floors.data ?? []).map((f) => ({
                    value: String(f.id),
                    label: `${f.buildingName} - ${f.label}`,
                  }))}
                  {...form.getInputProps('building_floor_id')}
                />
                <Select
                  label="유형"
                  withAsterisk
                  data={(spaceTypes.data ?? []).map((t) => ({ value: String(t.id), label: t.name }))}
                  {...form.getInputProps('space_type_id')}
                />
                <TextInput label="이름" withAsterisk placeholder="예: 201호" {...form.getInputProps('name')} />
                <NumberInput label="정렬 순서" {...form.getInputProps('sort_order')} />
                <Switch label="활성" {...form.getInputProps('is_active', { type: 'checkbox' })} />
              </>
            )}
          />
          {floors.data?.length === 0 && (
            <Alert color="yellow" mt="md">
              건물·층을 먼저 등록하면 공간을 추가할 수 있습니다.
            </Alert>
          )}
        </Tabs.Panel>

        <Tabs.Panel value="buildings">
          <BuildingsTab />
        </Tabs.Panel>

        <Tabs.Panel value="types">
          <CrudMaster<SpaceType, { name: string; is_course_eligible: boolean; sort_order: number; is_active: boolean }>
            addLabel="유형 추가"
            queryKey={['space-types']}
            fetchList={() => unwrap(api.GET('/api/v1/space-types'))}
            create={(v) =>
              unwrap(
                api.POST('/api/v1/space-types', {
                  body: { name: v.name, is_course_eligible: v.is_course_eligible, sort_order: v.sort_order },
                }),
              ).then(() => undefined)
            }
            update={(id, v) =>
              unwrap(
                api.PATCH('/api/v1/space-types/{space_type_id}', {
                  params: { path: { space_type_id: id } },
                  body: v,
                }),
              ).then(() => undefined)
            }
            columns={[
              { head: '이름', cell: (r) => r.name },
              {
                head: '강좌 개설',
                cell: (r) => (
                  <Badge color={r.is_course_eligible ? 'teal' : 'gray'}>
                    {r.is_course_eligible ? '가능' : '불가'}
                  </Badge>
                ),
              },
              { head: '정렬', cell: (r) => r.sort_order },
              { head: '상태', cell: (r) => activeBadge(r.is_active) },
            ]}
            initial={{ name: '', is_course_eligible: true, sort_order: 0, is_active: true }}
            toValues={(r) => ({
              name: r.name,
              is_course_eligible: r.is_course_eligible,
              sort_order: r.sort_order,
              is_active: r.is_active,
            })}
            validate={{ name: (v: string) => (v.trim() ? null : '이름을 입력하세요.') }}
            renderForm={(form) => (
              <>
                <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
                <Switch
                  label="강좌 개설 가능"
                  {...form.getInputProps('is_course_eligible', { type: 'checkbox' })}
                />
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

function BuildingsTab() {
  const [floorsBuilding, setFloorsBuilding] = useState<Building | null>(null)
  return (
    <>
      <CrudMaster<Building, { name: string; description: string; sort_order: number; is_active: boolean }>
        addLabel="건물 추가"
        queryKey={['buildings']}
        fetchList={() => unwrap(api.GET('/api/v1/buildings'))}
        create={(v) =>
          unwrap(
            api.POST('/api/v1/buildings', {
              body: { name: v.name, description: v.description || null, sort_order: v.sort_order },
            }),
          ).then(() => undefined)
        }
        update={(id, v) =>
          unwrap(
            api.PATCH('/api/v1/buildings/{building_id}', {
              params: { path: { building_id: id } },
              body: {
                name: v.name,
                description: v.description || null,
                sort_order: v.sort_order,
                is_active: v.is_active,
              },
            }),
          ).then(() => undefined)
        }
        columns={[
          { head: '이름', cell: (r) => r.name },
          { head: '설명', cell: (r) => r.description ?? '-' },
          { head: '상태', cell: (r) => activeBadge(r.is_active) },
        ]}
        initial={{ name: '', description: '', sort_order: 0, is_active: true }}
        toValues={(r) => ({
          name: r.name,
          description: r.description ?? '',
          sort_order: r.sort_order,
          is_active: r.is_active,
        })}
        validate={{ name: (v: string) => (v.trim() ? null : '이름을 입력하세요.') }}
        rowActions={(b) => (
          <Button size="xs" variant="light" onClick={() => setFloorsBuilding(b)}>
            층
          </Button>
        )}
        renderForm={(form) => (
          <>
            <TextInput label="이름" withAsterisk {...form.getInputProps('name')} />
            <TextInput label="설명" {...form.getInputProps('description')} />
            <NumberInput label="정렬 순서" {...form.getInputProps('sort_order')} />
            <Switch label="활성" {...form.getInputProps('is_active', { type: 'checkbox' })} />
          </>
        )}
      />
      <FloorsModal building={floorsBuilding} onClose={() => setFloorsBuilding(null)} />
    </>
  )
}

function FloorsModal({ building, onClose }: { building: Building | null; onClose: () => void }) {
  const qc = useQueryClient()
  const list = useQuery({
    queryKey: ['floors', building?.id],
    enabled: building !== null,
    queryFn: () =>
      unwrap(
        api.GET('/api/v1/buildings/{building_id}/floors', {
          params: { path: { building_id: building!.id } },
        }),
      ),
  })
  const form = useForm({ initialValues: { label: '', sort_order: 0 } })

  const invalidate = () => {
    qc.invalidateQueries({ queryKey: ['floors', building?.id] })
    qc.invalidateQueries({ queryKey: ['all-floors'] })
  }
  const add = useMutation({
    mutationFn: async (v: typeof form.values) => {
      await unwrap(
        api.POST('/api/v1/buildings/{building_id}/floors', {
          params: { path: { building_id: building!.id } },
          body: { label: v.label, sort_order: v.sort_order },
        }),
      )
    },
    onSuccess: () => {
      invalidate()
      form.reset()
    },
  })
  const remove = useMutation({
    mutationFn: async (f: Floor) => {
      await unwrap(
        api.DELETE('/api/v1/building-floors/{floor_id}', {
          params: { path: { floor_id: f.id } },
        }),
      )
    },
    onSuccess: invalidate,
  })

  return (
    <Modal opened={building !== null} onClose={onClose} title={`${building?.name} 층`}>
      <Stack>
        <Table>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>층</Table.Th>
              <Table.Th>정렬</Table.Th>
              <Table.Th />
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {list.data?.map((f) => (
              <Table.Tr key={f.id}>
                <Table.Td>{f.label}</Table.Td>
                <Table.Td>{f.sort_order}</Table.Td>
                <Table.Td>
                  <Button
                    size="xs"
                    color="red"
                    variant="light"
                    loading={remove.isPending && remove.variables?.id === f.id}
                    onClick={() => remove.mutate(f)}
                  >
                    삭제
                  </Button>
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
        {list.data?.length === 0 && (
          <Text c="dimmed" ta="center">
            등록된 층이 없습니다.
          </Text>
        )}
        <form onSubmit={form.onSubmit((v) => add.mutate(v))}>
          <Group align="flex-end">
            <TextInput label="층" placeholder="예: 2층" flex={1} {...form.getInputProps('label')} />
            <NumberInput label="정렬" w={90} {...form.getInputProps('sort_order')} />
            <Button type="submit" loading={add.isPending} disabled={!form.values.label}>
              추가
            </Button>
          </Group>
          {(add.isError || remove.isError) && (
            <Alert color="red" mt="xs">
              {errMessage(add.error ?? remove.error)}
            </Alert>
          )}
        </form>
      </Stack>
    </Modal>
  )
}
