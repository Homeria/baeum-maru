import type { UseQueryResult } from "@tanstack/react-query";
import type { PriorityReasonListResponse } from "../../../api/generated/PriorityReasonListResponse";
import styles from "../../../components/layout/listPage.module.css";
import tableStyles from "../../../components/table/table.module.css";
import { priorityMethodLabel } from "./priorityReasonFormData";

/** {@link PriorityReasonList}의 속성. */
export interface PriorityReasonListProps {
  reasons: UseQueryResult<PriorityReasonListResponse>;
  /** 고치는 중인 사유. 그 줄을 강조한다. */
  selectedId: number | null;
  /** 줄을 눌렀을 때. 바꿀 수 없는 역할이면 없다. */
  onSelect?: (id: number) => void;
}

/** 우선 배정 사유 목록. 이름 순서. */
export function PriorityReasonList({
  reasons,
  selectedId,
  onSelect,
}: PriorityReasonListProps) {
  const { data, isPending, isError, error } = reasons;

  if (isPending) {
    return <p className={tableStyles.message}>불러오는 중입니다…</p>;
  }
  if (isError) {
    return (
      <p className={tableStyles.error} role="alert">
        {error.message}
      </p>
    );
  }

  return (
    <section className={styles.list} aria-label="우선 배정 사유">
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="우선 배정 사유 목록">
          <thead>
            <tr>
              <th scope="col">이름</th>
              <th scope="col">배정 방식</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((reason) => (
              <tr
                key={reason.id}
                className={onSelect ? tableStyles.row : undefined}
                tabIndex={onSelect ? 0 : undefined}
                aria-current={reason.id === selectedId ? "true" : undefined}
                onClick={onSelect ? () => onSelect(reason.id) : undefined}
                onKeyDown={
                  onSelect
                    ? (event) => {
                        if (event.key === "Enter" || event.key === " ") {
                          event.preventDefault();
                          onSelect(reason.id);
                        }
                      }
                    : undefined
                }
              >
                <td>{reason.name}</td>
                <td>{priorityMethodLabel(reason.method)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {data.items.length === 0 && (
          <p className={tableStyles.message}>
            등록된 사유가 없습니다. 사유가 없으면 신청 탭에 우선 배정 칸이
            보이지 않고, 모든 신청을 추첨합니다
          </p>
        )}
      </div>
      <p className={styles.summary}>모두 {data.items.length}개</p>
    </section>
  );
}
