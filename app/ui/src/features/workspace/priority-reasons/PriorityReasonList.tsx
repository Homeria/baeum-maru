import type { UseQueryResult } from "@tanstack/react-query";
import type { PriorityReasonListResponse } from "../../../api/generated/PriorityReasonListResponse";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { Button } from "../../../components/ui/Button";
import { priorityMethodLabel } from "./priorityReasonFormData";
import styles from "./PriorityReasonsPage.module.css";

/** {@link PriorityReasonList}의 속성. */
export interface PriorityReasonListProps {
  reasons: UseQueryResult<PriorityReasonListResponse>;
  /** 고치는 중인 사유. 그 줄을 강조한다. */
  selectedId: number | null;
  /** 우선 배정 사유를 등록하거나 고칠 수 있는지 여부(관리자 전용). */
  canManage: boolean;
  onSelect?: (id: number) => void;
  onCreate?: () => void;
}

/** 전체 폭 우선 배정 사유 표. 이름 순서로 보여 준다. */
export function PriorityReasonList({
  reasons,
  selectedId,
  canManage,
  onSelect,
  onCreate,
}: PriorityReasonListProps) {
  const { data, isPending, isError, error, refetch } = reasons;
  const rows = data?.items ?? [];

  const columns: DataTableColumn<PriorityReasonResponse>[] = [
    {
      key: "name",
      header: "이름",
      isRowHeader: true,
      render: (reason) => reason.name,
    },
    {
      key: "method",
      header: "배정 방식",
      width: "220px",
      render: (reason) => priorityMethodLabel(reason.method),
    },
  ];

  if (canManage && onSelect) {
    columns.push({
      key: "actions",
      header: "",
      width: "96px",
      align: "right",
      isAction: true,
      render: (reason) => (
        <Button
          size="sm"
          variant="secondary"
          onClick={() => onSelect(reason.id)}
        >
          고치기
        </Button>
      ),
    });
  }

  return (
    <Card padding="none">
      <div className={styles.toolbar}>
        <div className={styles.headingGroup}>
          <h2 className={styles.title}>우선 배정 사유</h2>
          {data && <p className={styles.count}>모두 {rows.length}개</p>}
        </div>
        {canManage && onCreate && (
          <Button variant="primary" onClick={onCreate}>
            사유 등록
          </Button>
        )}
      </div>

      <DataTable
        caption="우선 배정 사유 목록"
        columns={columns}
        rows={rows}
        getRowKey={(reason) => reason.id}
        selectedKey={selectedId}
        onRowClick={
          canManage && onSelect ? (reason) => onSelect(reason.id) : undefined
        }
        loading={isPending}
        loadingText="우선 배정 사유를 불러오는 중입니다…"
        error={isError ? error.message : null}
        onRetry={() => void refetch()}
        emptyMessage="등록된 사유가 없습니다. 사유가 없으면 신청 탭에 우선 배정 칸이 보이지 않고, 모든 신청을 추첨합니다."
      />
    </Card>
  );
}
