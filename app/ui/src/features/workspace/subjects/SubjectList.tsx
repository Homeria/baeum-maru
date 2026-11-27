import type { UseQueryResult } from "@tanstack/react-query";
import type { SubjectListResponse } from "../../../api/generated/SubjectListResponse";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { Button } from "../../../components/ui/Button";
import { usesCategories } from "./subjectFormData";
import styles from "./SubjectsPage.module.css";

/** {@link SubjectList}의 속성. */
export interface SubjectListProps {
  subjects: UseQueryResult<SubjectListResponse>;
  /** 고치는 중인 과목. 그 줄을 강조한다. */
  selectedId: number | null;
  /** 과목을 등록하거나 고칠 수 있는지 여부. */
  canEdit: boolean;
  onSelect: (id: number) => void;
  onCreate: () => void;
}

/** 전체 폭 과목 표. 분류를 쓴 과목이 없으면 분류 열을 숨긴다. */
export function SubjectList({
  subjects,
  selectedId,
  canEdit,
  onSelect,
  onCreate,
}: SubjectListProps) {
  const { data, isPending, isError, error, refetch } = subjects;
  const rows = data?.items ?? [];
  const showCategory = usesCategories(rows);

  const columns: DataTableColumn<SubjectResponse>[] = [];
  if (showCategory) {
    columns.push({
      key: "category",
      header: "분류",
      width: "180px",
      render: (subject) => subject.category ?? "",
    });
  }
  columns.push({
    key: "name",
    header: "이름",
    isRowHeader: true,
    render: (subject) => subject.name,
  });
  if (canEdit) {
    columns.push({
      key: "actions",
      header: "",
      width: "96px",
      align: "right",
      isAction: true,
      render: (subject) => (
        <Button
          size="sm"
          variant="secondary"
          onClick={() => onSelect(subject.id)}
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
          <h2 className={styles.title}>과목 목록</h2>
          {data && <p className={styles.count}>모두 {rows.length}개</p>}
        </div>
        {canEdit && (
          <Button variant="primary" onClick={onCreate}>
            과목 등록
          </Button>
        )}
      </div>

      <DataTable
        caption="과목 목록"
        columns={columns}
        rows={rows}
        getRowKey={(subject) => subject.id}
        selectedKey={selectedId}
        onRowClick={canEdit ? (subject) => onSelect(subject.id) : undefined}
        loading={isPending}
        loadingText="과목 목록을 불러오는 중입니다…"
        error={isError ? error.message : null}
        onRetry={() => void refetch()}
        emptyMessage={
          canEdit
            ? "등록된 과목이 없습니다. 오른쪽 위의 과목 등록 버튼으로 등록해 주세요."
            : "등록된 과목이 없습니다."
        }
      />
    </Card>
  );
}
