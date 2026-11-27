import { useId, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import {
  listApplicationCounts,
  listCourses,
  listSelections,
} from "../courses/api";
import { courseName } from "../courses/courseFormData";
import {
  STAGE_LABELS,
  countLabel,
  countsByCourse,
  stagesByCourse,
} from "../courses/courseStatusData";
import { listTerms } from "../terms/api";
import { TermBar } from "../terms/TermBar";
import { currentTerm } from "../terms/termFormData";
import { LotteryBoard } from "./LotteryBoard";
import styles from "./SelectionPage.module.css";

/**
 * 추첨 탭. 학기와 강좌를 고르면 그 강좌의 3칸 추첨 보드가 나온다.
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
    return <NoticeBanner message="학기를 불러오는 중입니다…" />;
  }
  if (terms.isError) {
    return (
      <NoticeBanner
        tone="error"
        message="학기를 불러오지 못했습니다. 다시 시도해 주세요."
        actionLabel="다시 해 보기"
        onAction={() => void terms.refetch()}
      />
    );
  }
  if (!current) {
    return (
      <NoticeBanner message="아직 등록된 학기가 없습니다. 강좌 탭에서 학기와 강좌를 먼저 등록해 주세요." />
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
        <NoticeBanner
          tone="error"
          message={failure.message}
          actionLabel="다시 해 보기"
          onAction={() => {
            void courses.refetch();
            void counts.refetch();
            void selections.refetch();
          }}
        />
      ) : courses.data && counts.data && selections.data ? (
        <>
          <Card>
            <div className={styles.courseBar}>
              <label htmlFor={selectId} className={styles.courseLabel}>
                강좌
              </label>
              <select
                id={selectId}
                className={styles.courseSelect}
                value={courseId ?? ""}
                onChange={(event) =>
                  setCourseId(
                    event.target.value === ""
                      ? null
                      : Number(event.target.value),
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
          </Card>

          {course ? (
            <LotteryBoard
              // 강좌를 바꾸면 섞던 것과 창을 새로 시작
              key={course.id}
              termId={current.id}
              courseId={course.id}
              courseName={courseName(course)}
            />
          ) : (
            <NoticeBanner
              message={
                courses.data.items.length === 0
                  ? "이 학기에 등록된 강좌가 없습니다. 강좌 탭에서 먼저 등록해 주세요."
                  : "위에서 강좌를 고르면 신청자, 당첨자, 대기자가 나옵니다."
              }
            />
          )}
        </>
      ) : (
        <NoticeBanner message="강좌를 불러오는 중입니다…" />
      )}
    </div>
  );
}
