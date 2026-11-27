import type { UseQueryResult } from "@tanstack/react-query";
import type { StaffListResponse } from "../../api/generated/StaffListResponse";
import styles from "../../components/layout/listPage.module.css";
import tableStyles from "../../components/table/table.module.css";
import { ROLE_LABELS } from "../gate/auth/roles";
import { todayText } from "../../lib/dates";
import { staffStatusLabel } from "./staffFormData";

/** {@link StaffList}의 속성. */
export interface StaffListProps {
  staff: UseQueryResult<StaffListResponse>;
  /** 고치는 중인 직원. 그 줄을 강조한다. */
  selectedId: number | null;
  onSelect: (id: number) => void;
}

/** 직원 목록. 사용 중인 직원 먼저. 줄을 누르면 그 직원을 고른다. */
export function StaffList({ staff, selectedId, onSelect }: StaffListProps) {
  const { data, isPending, isError, error } = staff;

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

  const today = todayText();

  return (
    <section className={styles.list} aria-label="직원">
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="직원 목록">
          <thead>
            <tr>
              <th scope="col">이름</th>
              <th scope="col">역할</th>
              <th scope="col">유효기간</th>
              <th scope="col">상태</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((item) => (
              <tr
                key={item.id}
                className={tableStyles.row}
                tabIndex={0}
                aria-current={item.id === selectedId ? "true" : undefined}
                onClick={() => onSelect(item.id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(item.id);
                  }
                }}
              >
                <td>{item.name}</td>
                <td>{ROLE_LABELS[item.role]}</td>
                <td>{item.validUntil ? `${item.validUntil}까지` : ""}</td>
                <td>{staffStatusLabel(item, today)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className={styles.summary}>모두 {data.items.length}명</p>
    </section>
  );
}
