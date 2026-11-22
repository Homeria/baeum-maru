import type { CourseResponse } from "../../api/generated/CourseResponse";
import listStyles from "../../components/layout/listPage.module.css";
import tableStyles from "../../components/table/table.module.css";
import { scheduleSummary } from "../courses/courseFormData";
import { countLabel } from "./applicationsData";

/** {@link CourseChoiceList}의 속성. */
export interface CourseChoiceListProps {
  courses: readonly CourseResponse[];
  /** 강좌 id로 찾는 신청 인원. 없으면 0명. */
  counts: ReadonlyMap<number, number>;
  /** 강의실 이름에 건물을 붙일지. 건물이 여럿일 때만. */
  showBuilding: boolean;
}

/** 이 학기의 강좌와 신청 인원/정원. */
export function CourseChoiceList({
  courses,
  counts,
  showBuilding,
}: CourseChoiceListProps) {
  return (
    <section className={listStyles.list} aria-label="강좌">
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table} aria-label="신청할 강좌">
          <thead>
            <tr>
              <th scope="col">과목</th>
              <th scope="col">수준</th>
              <th scope="col">반</th>
              <th scope="col">수업 시간</th>
              <th scope="col">강의실</th>
              <th scope="col">신청/정원</th>
            </tr>
          </thead>
          <tbody>
            {courses.map((course) => {
              const summary = scheduleSummary(course.schedules, showBuilding);
              return (
                <tr key={course.id}>
                  <td>{course.subjectName}</td>
                  <td>{course.level ?? ""}</td>
                  <td>{course.section ?? ""}</td>
                  <td>{summary.times}</td>
                  <td>{summary.room}</td>
                  <td>
                    {countLabel(counts.get(course.id) ?? 0, course.capacity)}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {courses.length === 0 && (
          <p className={tableStyles.message}>
            이 학기에 등록된 강좌가 없습니다. 강좌 탭에서 먼저 등록해 주세요
          </p>
        )}
      </div>
    </section>
  );
}
