import type { UseQueryResult } from "@tanstack/react-query";
import type { CourseListResponse } from "../../../api/generated/CourseListResponse";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { Button } from "../../../components/ui/Button";
import { scheduleSummary } from "./courseFormData";
import styles from "./CourseList.module.css";

/** {@link CourseList}의 속성. */
export interface CourseListProps {
  courses: UseQueryResult<CourseListResponse>;
  /** 강의실 이름에 건물을 붙일지. 건물이 여럿일 때만. */
  showBuilding: boolean;
  /** 고치는 중인 강좌. 그 줄을 강조한다. */
  selectedId: number | null;
  /** 강좌를 등록하거나 고칠 수 있는지 여부. */
  canEdit: boolean;
  onSelect: (id: number) => void;
  onCreate: () => void;
}

/**
 * 한 학기의 전체 폭 강좌 표.
 *
 * 여러 요일·시간대가 있는 수업 시간은 셀 안에서 줄바꿈해 가독성을 높인다.
 */
export function CourseList({
  courses,
  showBuilding,
  selectedId,
  canEdit,
  onSelect,
  onCreate,
}: CourseListProps) {
  const { data, isPending, isError, error, refetch } = courses;
  const rows = data?.items ?? [];

  const columns: DataTableColumn<CourseResponse>[] = [
    {
      key: "subjectName",
      header: "과목",
      isRowHeader: true,
      render: (course) => course.subjectName,
    },
    {
      key: "level",
      header: "수준",
      width: "96px",
      render: (course) => course.level ?? "",
    },
    {
      key: "section",
      header: "반",
      width: "96px",
      render: (course) => course.section ?? "",
    },
    {
      key: "capacity",
      header: "정원",
      width: "84px",
      align: "right",
      render: (course) => `${course.capacity}명`,
    },
    {
      key: "schedules",
      header: "수업 시간",
      render: (course) => {
        const summary = scheduleSummary(course.schedules, showBuilding);
        const lines = summary.times
          .split(", ")
          .filter((line) => line.length > 0);
        return (
          <div className={styles.scheduleLines}>
            {lines.map((line) => (
              <span key={line} className={styles.scheduleLine}>
                {line}
              </span>
            ))}
          </div>
        );
      },
    },
    {
      key: "room",
      header: "강의실",
      render: (course) => scheduleSummary(course.schedules, showBuilding).room,
    },
  ];

  if (canEdit) {
    columns.push({
      key: "actions",
      header: "",
      width: "96px",
      align: "right",
      isAction: true,
      render: (course) => (
        <Button
          size="sm"
          variant="secondary"
          onClick={() => onSelect(course.id)}
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
          <h2 className={styles.title}>강좌 목록</h2>
          {data && <p className={styles.count}>모두 {rows.length}개</p>}
        </div>
        {canEdit && (
          <Button variant="primary" onClick={onCreate}>
            강좌 등록
          </Button>
        )}
      </div>

      <DataTable
        caption="강좌 목록"
        columns={columns}
        rows={rows}
        getRowKey={(course) => course.id}
        selectedKey={selectedId}
        onRowClick={canEdit ? (course) => onSelect(course.id) : undefined}
        loading={isPending}
        loadingText="강좌 목록을 불러오는 중입니다…"
        error={isError ? error.message : null}
        onRetry={() => void refetch()}
        emptyMessage={
          canEdit
            ? "이 학기에 등록된 강좌가 없습니다. 오른쪽 위의 강좌 등록 버튼으로 등록해 주세요."
            : "이 학기에 등록된 강좌가 없습니다."
        }
      />
    </Card>
  );
}
