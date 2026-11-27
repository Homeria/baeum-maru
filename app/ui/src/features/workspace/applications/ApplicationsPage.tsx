import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Card } from "../../../components/surface/Card";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { todayText } from "../../../lib/dates";
import { useCan } from "../../gate/auth/useCan";
import {
  listApplicationCounts,
  listCourses,
  listSelections,
} from "../courses/api";
import { countsByCourse, stagesByCourse } from "../courses/courseStatusData";
import { MemberList } from "../members/MemberList";
import { listPriorityReasons } from "../priority-reasons/api";
import { listRooms } from "../rooms/api";
import { hasSeveralBuildings } from "../rooms/roomFormData";
import { listClosedCourses } from "../selection-results/api";
import { listTerms } from "../terms/api";
import { TermBar } from "../terms/TermBar";
import { currentTerm } from "../terms/termFormData";
import { listApplications } from "./api";
import {
  additionalByCourse,
  isTermEnded,
  toggleChoice,
  togglePriority,
  type ChosenPriorities,
} from "./applicationsData";
import styles from "./ApplicationsPage.module.css";
import { ApplyBar } from "./ApplyBar";
import { CourseChoiceList } from "./CourseChoiceList";
import { MemberApplications } from "./MemberApplications";

/**
 * 신청 탭. 맨 위에서 학기를 고르고, 왼쪽에서 회원을 찾아 고르면 그 회원의 신청이 보인다.
 * 오른쪽은 그 학기의 강좌와 신청 인원/정원 표, 그리고 아래 신청 막대다.
 *
 * 선발 마감한 강좌는 학기 종료일까지 추가 신청으로 고른다(BR-X1). 고르기 전에 바로 당첨인지
 * 대기 몇 번째인지 꼬리표로 보여 준다(BR-X2).
 * 신청을 받을 수 없는 역할(BR-K3)에게는 강좌 선택·신청 막대를 숨긴다.
 */
export function ApplicationsPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedTermId, setSelectedTermId] = useState<number | null>(null);
  const [memberId, setMemberId] = useState<number | null>(null);
  const [chosen, setChosen] = useState<readonly number[]>([]);
  const [priorities, setPriorities] = useState<ChosenPriorities>(new Map());
  const canTake = useCan("takeApplications");
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
  // 추첨한 강좌는 고르지 못하게 (BR-L3)
  const selections = useQuery({
    queryKey: ["selections", termId],
    queryFn: () => listSelections(termId ?? 0),
    enabled: termId !== null,
  });
  // 마감한 강좌에 추가 신청을 받으면 어떻게 되는지 미리 보기 (BR-X2)
  const closedCourses = useQuery({
    queryKey: ["closed-courses", termId],
    queryFn: () => listClosedCourses(termId ?? 0),
    enabled: termId !== null,
  });
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });
  const reasons = useQuery({
    queryKey: ["priority-reasons"],
    queryFn: listPriorityReasons,
  });
  // MemberApplications와 같은 키라 한 번만 받음
  const applications = useQuery({
    queryKey: ["applications", termId, memberId],
    queryFn: () => listApplications(termId ?? 0, memberId ?? 0),
    enabled: termId !== null && memberId !== null,
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

  const showBuilding = rooms.data
    ? hasSeveralBuildings(rooms.data.items)
    : false;
  const reasonItems = reasons.data?.items ?? [];
  const stages = stagesByCourse(selections.data?.items ?? []);
  const additional = additionalByCourse(closedCourses.data?.items ?? []);
  const termEnded = isTermEnded(current, todayText());
  const clearChoices = () => {
    setChosen([]);
    setPriorities(new Map());
  };
  const appliedIds = new Set(
    applications.data?.items.map((application) => application.courseId) ?? [],
  );

  return (
    <div className={styles.page}>
      <TermBar
        terms={terms.data.items}
        current={current}
        onSelect={(id) => {
          setSelectedTermId(id);
          clearChoices();
        }}
        notice={null}
        onNotice={() => undefined}
        canEdit={false}
      />

      <div className={styles.content}>
        <div className={styles.memberSide}>
          <MemberList
            selectedId={memberId}
            onSelect={(id) => {
              setMemberId(id);
              clearChoices();
            }}
          />

          {memberId === null ? (
            <Card title="고른 회원의 신청">
              <NoticeBanner message="신청을 받을 회원을 위 목록에서 골라 주세요." />
            </Card>
          ) : courses.data ? (
            <MemberApplications
              termId={current.id}
              memberId={memberId}
              courses={courses.data.items}
              reasons={reasonItems}
              stages={stages}
            />
          ) : null}
        </div>

        <div className={styles.courseSide}>
          {courses.isError ? (
            <NoticeBanner
              tone="error"
              message={courses.error.message}
              actionLabel="다시 해 보기"
              onAction={() => void courses.refetch()}
            />
          ) : courses.data && counts.data ? (
            <>
              <CourseChoiceList
                courses={courses.data.items}
                counts={countsByCourse(counts.data.items)}
                showBuilding={showBuilding}
                selectable={canTake && memberId !== null}
                chosen={chosen}
                appliedIds={appliedIds}
                stages={stages}
                additional={additional}
                termEnded={termEnded}
                onToggle={(id) => {
                  setChosen((prev) => toggleChoice(prev, id));
                  // 빼는 강좌의 사유도 지운다. 다시 고르면 처음부터
                  setPriorities((prev) => {
                    const next = new Map(prev);
                    next.delete(id);
                    return next;
                  });
                }}
              />

              {canTake && (
                <ApplyBar
                  // 학기·회원이 바뀌면 알림과 창을 새로 시작
                  key={`${current.id}-${memberId ?? "none"}`}
                  termId={current.id}
                  memberId={memberId}
                  chosen={chosen}
                  courses={courses.data.items}
                  reasons={reasonItems}
                  priorities={priorities}
                  onTogglePriority={(courseId, reasonId) =>
                    setPriorities((prev) =>
                      togglePriority(prev, courseId, reasonId),
                    )
                  }
                  additional={additional}
                  onSubmitted={clearChoices}
                />
              )}
            </>
          ) : (
            <NoticeBanner message="강좌를 불러오는 중입니다…" />
          )}
        </div>
      </div>
    </div>
  );
}
