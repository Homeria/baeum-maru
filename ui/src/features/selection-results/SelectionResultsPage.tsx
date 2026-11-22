import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listCourses } from "../../api/courses";
import { listClosedCourses } from "../../api/selectionResults";
import { listTerms } from "../../api/terms";
import formStyles from "../../components/form/form.module.css";
import tableStyles from "../../components/table/table.module.css";
import { courseName } from "../courses/courseFormData";
import { TermBar } from "../terms/TermBar";
import { currentTerm } from "../terms/termFormData";
import { CourseResultBoard } from "./CourseResultBoard";
import { openSeatsLabel } from "./resultsData";
import styles from "./SelectionResultsPage.module.css";

/**
 * 선발 결과 탭. 선발을 마감한 강좌에서 포기와 대기자 승격을 처리한다(BR-C3, BR-W).
 *
 * 왼쪽은 마감한 강좌와 빈자리, 오른쪽은 고른 강좌의 당첨자·대기자·포기다.
 * 학기를 바꾸면 고른 강좌를 비운다.
 */
export function SelectionResultsPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedTermId, setSelectedTermId] = useState<number | null>(null);
  const [courseId, setCourseId] = useState<number | null>(null);
  const current = terms.data
    ? currentTerm(terms.data.items, selectedTermId)
    : null;
  const termId = current?.id ?? null;

  const courses = useQuery({
    queryKey: ["courses", termId],
    // enabled가 학기 없을 때 막으므로 0으로 부를 일은 없음
    queryFn: () => listCourses(termId ?? 0),
    enabled: termId !== null,
  });
  const closed = useQuery({
    queryKey: ["closed-courses", termId],
    queryFn: () => listClosedCourses(termId ?? 0),
    enabled: termId !== null,
  });

  if (terms.isPending) {
    return <p className={formStyles.loading}>학기를 불러오는 중입니다…</p>;
  }
  if (terms.isError) {
    return (
      <p className={formStyles.formError} role="alert">
        학기를 불러오지 못했습니다. 다시 열어 주세요.
      </p>
    );
  }
  if (!current) {
    return (
      <p className={styles.empty}>
        아직 등록된 학기가 없습니다. 강좌 탭에서 학기와 강좌를 먼저 등록해
        주세요.
      </p>
    );
  }

  const failure = courses.error ?? closed.error;
  const nameOf = (id: number) => {
    const course = courses.data?.items.find((candidate) => candidate.id === id);
    return course ? courseName(course) : "지워진 강좌";
  };
  const chosen = closed.data?.items.find((item) => item.courseId === courseId);

  return (
    <div className={styles.page}>
      <TermBar
        terms={terms.data.items}
        current={current}
        onSelect={(id) => {
          setSelectedTermId(id);
          setCourseId(null);
        }}
        notice={null}
        onNotice={() => undefined}
        canEdit={false}
      />
      {failure ? (
        <p className={formStyles.formError} role="alert">
          {failure.message}
        </p>
      ) : courses.data && closed.data ? (
        <div className={styles.content}>
          <section className={styles.courses} aria-label="마감한 강좌">
            <div className={tableStyles.tableWrap}>
              <table
                className={tableStyles.table}
                aria-label="마감한 강좌 목록"
              >
                <thead>
                  <tr>
                    <th scope="col">강좌</th>
                    <th scope="col">당첨/정원</th>
                    <th scope="col">대기</th>
                    <th scope="col">빈자리</th>
                  </tr>
                </thead>
                <tbody>
                  {closed.data.items.map((item) => (
                    <tr
                      key={item.courseId}
                      className={tableStyles.row}
                      tabIndex={0}
                      aria-current={
                        item.courseId === courseId ? "true" : undefined
                      }
                      onClick={() => setCourseId(item.courseId)}
                      onKeyDown={(event) => {
                        if (event.key === "Enter" || event.key === " ") {
                          event.preventDefault();
                          setCourseId(item.courseId);
                        }
                      }}
                    >
                      <td>{nameOf(item.courseId)}</td>
                      <td>
                        {item.selected}/{item.capacity}
                      </td>
                      <td>{item.waitlisted}</td>
                      <td>{openSeatsLabel(item.openSeats)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {closed.data.items.length === 0 && (
                <p className={tableStyles.message}>
                  선발을 마감한 강좌가 없습니다. 추첨 탭에서 추첨하고 마감하면
                  여기에 나옵니다
                </p>
              )}
            </div>
          </section>
          {chosen ? (
            <CourseResultBoard
              // 강좌를 바꾸면 검색 칸을 새로 시작
              key={chosen.courseId}
              courseId={chosen.courseId}
              courseName={nameOf(chosen.courseId)}
            />
          ) : (
            <p className={styles.hint}>
              왼쪽에서 강좌를 고르면 당첨자, 대기자, 포기한 사람이 나옵니다.
              포기와 대기자 연락 결과를 여기서 남깁니다.
            </p>
          )}
        </div>
      ) : (
        <p className={formStyles.loading}>강좌를 불러오는 중입니다…</p>
      )}
    </div>
  );
}
