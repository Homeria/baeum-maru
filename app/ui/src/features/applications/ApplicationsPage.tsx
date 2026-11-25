import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  listApplicationCounts,
  listApplications,
} from "../../api/applications";
import { listCourses } from "../../api/courses";
import { listSelections } from "../../api/lotteries";
import { listPriorityReasons } from "../../api/priorityReasons";
import { listRooms } from "../../api/rooms";
import { listClosedCourses } from "../../api/selectionResults";
import { listTerms } from "../../api/terms";
import formStyles from "../../components/form/form.module.css";
import { ViewOnlyNotice } from "../auth/ViewOnlyNotice";
import { MemberList } from "../members/MemberList";
import { hasSeveralBuildings } from "../rooms/roomFormData";
import { TermBar } from "../terms/TermBar";
import { currentTerm } from "../terms/termFormData";
import { todayText } from "../staff/staffFormData";
import { useCan } from "../auth/useCan";
import { stagesByCourse } from "../selection/selectionData";
import {
  additionalByCourse,
  countsByCourse,
  isTermEnded,
  toggleChoice,
  togglePriority,
  type ChosenPriorities,
} from "./applicationsData";
import { ApplyBar } from "./ApplyBar";
import { CourseChoiceList } from "./CourseChoiceList";
import { MemberApplications } from "./MemberApplications";
import styles from "./ApplicationsPage.module.css";

/**
 * 신청 탭. 맨 위에서 학기를 고르고, 왼쪽에서 회원을 찾아 고르면 그 회원의 신청이 보인다.
 * 오른쪽은 그 학기의 강좌와 신청 인원/정원이다. 강좌를 골라 한 번에 신청한다.
 *
 * 선발 마감한 강좌는 학기 종료일까지 추가 신청으로 고른다(BR-X1). 고르기 전에 바로 당첨인지
 * 대기 몇 번째인지 보여 준다(BR-X2).
 *
 * 학기나 회원을 바꾸면 고른 강좌를 비운다. 학기 등록·고치기는 강좌 탭에서 한다.
 * 신청을 받을 수 없는 역할(BR-K3)은 강좌를 고를 수 없다.
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
            <p className={styles.hint}>
              신청을 받을 회원을 목록에서 골라 주세요.
            </p>
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
        {courses.isError ? (
          <p className={formStyles.formError} role="alert">
            {courses.error.message}
          </p>
        ) : courses.data && counts.data ? (
          <div className={styles.courseSide}>
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
            {canTake ? (
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
            ) : (
              <ViewOnlyNotice />
            )}
          </div>
        ) : (
          <p className={formStyles.loading}>강좌를 불러오는 중입니다…</p>
        )}
      </div>
    </div>
  );
}
