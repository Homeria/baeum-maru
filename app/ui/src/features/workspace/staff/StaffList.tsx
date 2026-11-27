import type { UseQueryResult } from "@tanstack/react-query";
import type { StaffListResponse } from "../../../api/generated/StaffListResponse";
import type { StaffResponse } from "../../../api/generated/StaffResponse";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { Button } from "../../../components/ui/Button";
import {
  StatusPill,
  type StatusPillTone,
} from "../../../components/ui/StatusPill";
import { todayText } from "../../../lib/dates";
import { ROLE_LABELS } from "../../gate/auth/roles";
import { staffStatusLabel } from "./staffFormData";
import styles from "./StaffPage.module.css";

/** {@link StaffList}의 속성. */
export interface StaffListProps {
  staff: UseQueryResult<StaffListResponse>;
  /** 고치는 중인 직원. 그 줄을 강조한다. */
  selectedId: number | null;
  onSelect: (id: number) => void;
  onCreate: () => void;
  onResetPin: (staff: StaffResponse) => void;
}

function statusTone(staff: StaffResponse, today: string): StatusPillTone {
  if (staff.disabled) {
    return "neutral";
  }
  if (staff.validUntil !== null && staff.validUntil < today) {
    return "neutral";
  }
  if (staff.locked) {
    return "bad";
  }
  return "ok";
}

/** 전체 폭 직원 표. 상태는 색상과 글자가 함께 담긴 꼬리표로 보여 준다. */
export function StaffList({
  staff,
  selectedId,
  onSelect,
  onCreate,
  onResetPin,
}: StaffListProps) {
  const { data, isPending, isError, error, refetch } = staff;
  const rows = data?.items ?? [];
  const today = todayText();

  const columns: DataTableColumn<StaffResponse>[] = [
    {
      key: "name",
      header: "이름",
      isRowHeader: true,
      render: (item) => item.name,
    },
    {
      key: "role",
      header: "역할",
      width: "140px",
      render: (item) => ROLE_LABELS[item.role],
    },
    {
      key: "validUntil",
      header: "유효기간",
      width: "160px",
      render: (item) =>
        item.validUntil ? `${item.validUntil}까지` : "기한 없음",
    },
    {
      key: "status",
      header: "상태",
      width: "120px",
      render: (item) => (
        <StatusPill tone={statusTone(item, today)}>
          {staffStatusLabel(item, today)}
        </StatusPill>
      ),
    },
    {
      key: "actions",
      header: "",
      width: "220px",
      align: "right",
      isAction: true,
      render: (item) => (
        <div className={styles.rowActions}>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => onResetPin(item)}
          >
            PIN 새로 정하기
          </Button>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => onSelect(item.id)}
          >
            고치기
          </Button>
        </div>
      ),
    },
  ];

  return (
    <Card padding="none">
      <div className={styles.toolbar}>
        <div className={styles.headingGroup}>
          <h2 className={styles.title}>직원 목록</h2>
          {data && <p className={styles.count}>모두 {rows.length}명</p>}
        </div>
        <Button variant="primary" onClick={onCreate}>
          직원 등록
        </Button>
      </div>

      <DataTable
        caption="직원 목록"
        columns={columns}
        rows={rows}
        getRowKey={(item) => item.id}
        selectedKey={selectedId}
        onRowClick={(item) => onSelect(item.id)}
        loading={isPending}
        loadingText="직원 목록을 불러오는 중입니다…"
        error={isError ? error.message : null}
        onRetry={() => void refetch()}
        emptyMessage="등록된 직원이 없습니다."
      />
    </Card>
  );
}
