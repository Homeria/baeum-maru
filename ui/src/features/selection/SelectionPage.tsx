import { useId, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listApplicationCounts } from "../../api/applications";
import { listCourses } from "../../api/courses";
import { listSelections } from "../../api/lotteries";
import { listTerms } from "../../api/terms";
import formStyles from "../../components/form/form.module.css";
import { countLabel, countsByCourse } from "../applications/applicationsData";
import { courseName } from "../courses/courseFormData";
import { TermBar } from "../terms/TermBar";
import { currentTerm } from "../terms/termFormData";
import { LotteryBoard } from "./LotteryBoard";
import { STAGE_LABELS, stagesByCourse } from "./selectionData";
import styles from "./SelectionPage.module.css";

/**
 * 추첨 탭. 학기와 강좌를 고르면 그 강좌의 추첨 화면이 나온다.
 *
 * 강좌 목록에는 신청 인원/정원과 선발 단계를 붙여 어느 강좌가 남았는지 보이게 한다.
 * 학기를 바꾸면 고른 강좌를 비운다.
 */
export function SelectionPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedTermId, setSelectedTermId] = useState<number | null>(null);
  const [courseId, setCourseId] = useState<number | null>(null);
  const selectId = useId();
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
  const counts = useQuery({
    queryKey: ["applicationCounts", termId],
    queryFn: () => listApplicationCounts(termId ?? 0),
    enabled: termId !== null,
  });
  const selections = useQuery({
    queryKey: ["selections", termId],
    queryFn: () => listSelections(termId ?? 0),
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

  const failure = courses.error ?? counts.error ?? selections.error;
  const countMap = countsByCourse(counts.data?.items ?? []);
  const stageMap = stagesByCourse(selections.data?.items ?? []);
  const course = courses.data?.items.find(
    (candidate) => candidate.id === courseId,
  );

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
      ) : courses.data && counts.data && selections.data ? (
        <>
          <div className={styles.courseBar}>
            <label htmlFor={selectId} className={formStyles.label}>
              강좌
            </label>
            <select
              id={selectId}
              className={`${formStyles.input} ${styles.courseSelect}`}
              value={courseId ?? ""}
              onChange={(event) =>
                setCourseId(
                  event.target.value === "" ? null : Number(event.target.value),
                )
              }
            >
              <option value="">추첨할 강좌를 골라 주세요</option>
              {courses.data.items.map((item) => {
                const count = countMap.get(item.id) ?? 0;
                const stage = stageMap.get(item.id) ?? "not_drawn";
                return (
                  <option key={item.id} value={item.id}>
                    {`${courseName(item)} · ${countLabel(count, item.capacity)} · ${STAGE_LABELS[stage]}`}
                  </option>
                );
              })}
            </select>
          </div>
          {course ? (
            <LotteryBoard
              // 강좌를 바꾸면 섞던 것과 창을 새로 시작
              key={course.id}
              termId={current.id}
              courseId={course.id}
              courseName={courseName(course)}
            />
          ) : (
            <p className={styles.hint}>
              {courses.data.items.length === 0
                ? "이 학기에 등록된 강좌가 없습니다. 강좌 탭에서 먼저 등록해 주세요."
                : "위에서 강좌를 고르면 신청자, 당첨자, 대기자가 나옵니다."}
            </p>
          )}
        </>
      ) : (
        <p className={formStyles.loading}>강좌를 불러오는 중입니다…</p>
      )}
    </div>
  );
}
