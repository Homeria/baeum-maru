import type { UseQueryResult } from "@tanstack/react-query";
import type { CourseListResponse } from "../../../api/generated/CourseListResponse";
import styles from "../../../components/layout/listPage.module.css";
import tableStyles from "../../../components/table/table.module.css";
import { scheduleSummary } from "./courseFormData";

/** {@link CourseList}의 속성. */
export interface CourseListProps {
  courses: UseQueryResult<CourseListResponse>;
  /** 강의실 이름에 건물을 붙일지. 건물이 여럿일 때만. */
  showBuilding: boolean;
  /** 고치는 중인 강좌. 그 줄을 강조한다. */
  selectedId: number | null;
  onSelect: (id: number) => void;
}

/**
 * 한 학기의 강좌 목록. 줄을 누르면 그 강좌를 고른다.
 *
 * 수업 시간은 `월·수 10:00~11:00`처럼 시각이 같은 요일을 묶어 보여 준다.
 */
export function CourseList({
  courses,
  showBuilding,
  selectedId,
  onSelect,
}: CourseListProps) {
  const { data, isPending, isError, error } = courses;

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
    <section className={styles.list} aria-label="강좌">
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="강좌 목록">
          <thead>
            <tr>
              <th scope="col">과목</th>
              <th scope="col">수준</th>
              <th scope="col">반</th>
              <th scope="col">정원</th>
              <th scope="col">수업 시간</th>
              <th scope="col">강의실</th>
            </tr>
          </thead>
          <tbody>
            {data.items.map((course) => {
              const summary = scheduleSummary(course.schedules, showBuilding);
              return (
                <tr
                  key={course.id}
                  className={tableStyles.row}
                  tabIndex={0}
                  aria-current={course.id === selectedId ? "true" : undefined}
                  onClick={() => onSelect(course.id)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      onSelect(course.id);
                    }
                  }}
                >
                  <td>{course.subjectName}</td>
                  <td>{course.level ?? ""}</td>
                  <td>{course.section ?? ""}</td>
                  <td>{course.capacity}명</td>
                  <td>{summary.times}</td>
                  <td>{summary.room}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {data.items.length === 0 && (
          <p className={tableStyles.message}>
            이 학기에 등록된 강좌가 없습니다. 오른쪽 칸에서 등록해 주세요
          </p>
        )}
      </div>
      <p className={styles.summary}>모두 {data.items.length}개</p>
    </section>
  );
}
