import type { UseQueryResult } from "@tanstack/react-query";
import type { RoomListResponse } from "../../../api/generated/RoomListResponse";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { Button } from "../../../components/ui/Button";
import { floorLabel } from "./floor";
import { hasSeveralBuildings } from "./roomFormData";
import styles from "./RoomsPage.module.css";

/** {@link RoomList}의 속성. */
export interface RoomListProps {
  rooms: UseQueryResult<RoomListResponse>;
  /** 고치는 중인 강의실. 그 줄을 강조한다. */
  selectedId: number | null;
  /** 강의실을 등록하거나 고칠 수 있는지 여부. */
  canEdit: boolean;
  onSelect: (id: number) => void;
  onCreate: () => void;
}

/** 전체 폭 강의실 표. 건물이 하나뿐이면 건물 열을 숨긴다. */
export function RoomList({
  rooms,
  selectedId,
  canEdit,
  onSelect,
  onCreate,
}: RoomListProps) {
  const { data, isPending, isError, error, refetch } = rooms;
  const rows = data?.items ?? [];
  const showBuilding = hasSeveralBuildings(rows);

  const columns: DataTableColumn<RoomResponse>[] = [];
  if (showBuilding) {
    columns.push({
      key: "building",
      header: "건물",
      width: "160px",
      render: (room) => room.building,
    });
  }
  columns.push(
    {
      key: "floor",
      header: "층",
      width: "120px",
      render: (room) => floorLabel(room.floor),
    },
    {
      key: "name",
      header: "이름",
      isRowHeader: true,
      render: (room) => room.name,
    },
  );
  if (canEdit) {
    columns.push({
      key: "actions",
      header: "",
      width: "96px",
      align: "right",
      isAction: true,
      render: (room) => (
        <Button size="sm" variant="secondary" onClick={() => onSelect(room.id)}>
          고치기
        </Button>
      ),
    });
  }

  return (
    <Card padding="none">
      <div className={styles.toolbar}>
        <div className={styles.headingGroup}>
          <h2 className={styles.title}>강의실 목록</h2>
          {data && <p className={styles.count}>모두 {rows.length}개</p>}
        </div>
        {canEdit && (
          <Button variant="primary" onClick={onCreate}>
            강의실 등록
          </Button>
        )}
      </div>

      <DataTable
        caption="강의실 목록"
        columns={columns}
        rows={rows}
        getRowKey={(room) => room.id}
        selectedKey={selectedId}
        onRowClick={canEdit ? (room) => onSelect(room.id) : undefined}
        loading={isPending}
        loadingText="강의실 목록을 불러오는 중입니다…"
        error={isError ? error.message : null}
        onRetry={() => void refetch()}
        emptyMessage={
          canEdit
            ? "등록된 강의실이 없습니다. 오른쪽 위의 강의실 등록 버튼으로 등록해 주세요."
            : "등록된 강의실이 없습니다."
        }
      />
    </Card>
  );
}
