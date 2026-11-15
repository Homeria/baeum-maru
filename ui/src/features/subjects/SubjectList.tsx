import type { UseQueryResult } from "@tanstack/react-query";
import type { SubjectListResponse } from "../../api/generated/SubjectListResponse";
import styles from "../../components/layout/listPage.module.css";
import tableStyles from "../../components/table/table.module.css";
import { usesCategories } from "./subjectFormData";

/** {@link SubjectList}의 속성. */
export interface SubjectListProps {
  subjects: UseQueryResult<SubjectListResponse>;
  /** 고치는 중인 과목. 그 줄을 강조한다. */
  selectedId: number | null;
  onSelect: (id: number) => void;
}

/** 과목 목록. 분류를 쓴 과목이 없으면 분류 열을 숨긴다. 줄을 누르면 그 과목을 고른다. */
export function SubjectList({
  subjects,
  selectedId,
  onSelect,
}: SubjectListProps) {
  const { data, isPending, isError, error } = subjects;

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

  const showCategory = usesCategories(data.items);

  return (
    <section className={styles.list} aria-label="과목">
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="과목 목록">
          <thead>
            <tr>
              {showCategory && <th scope="col">분류</th>}
              <th scope="col">이름</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((subject) => (
              <tr
                key={subject.id}
                className={tableStyles.row}
                tabIndex={0}
                aria-current={subject.id === selectedId ? "true" : undefined}
                onClick={() => onSelect(subject.id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(subject.id);
                  }
                }}
              >
                {showCategory && <td>{subject.category ?? ""}</td>}
                <td>{subject.name}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {data.items.length === 0 && (
          <p className={tableStyles.message}>
            등록된 과목이 없습니다. 오른쪽 칸에서 등록해 주세요
          </p>
        )}
      </div>
      <p className={styles.summary}>모두 {data.items.length}개</p>
    </section>
  );
}
