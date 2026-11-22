import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listApplicationCounts } from "../../api/applications";
import { listCourses } from "../../api/courses";
import { listRooms } from "../../api/rooms";
import { listTerms } from "../../api/terms";
import formStyles from "../../components/form/form.module.css";
import { MemberList } from "../members/MemberList";
import { hasSeveralBuildings } from "../rooms/roomFormData";
import { TermBar } from "../terms/TermBar";
import { currentTerm } from "../terms/termFormData";
import { countsByCourse } from "./applicationsData";
import { CourseChoiceList } from "./CourseChoiceList";
import { MemberApplications } from "./MemberApplications";
import styles from "./ApplicationsPage.module.css";

/**
 * 신청 탭. 맨 위에서 학기를 고르고, 왼쪽에서 회원을 찾아 고르면 그 회원의 신청이 보인다.
 * 오른쪽은 그 학기의 강좌와 신청 인원/정원이다.
 *
 * 학기 등록·고치기는 강좌 탭에서 한다.
 */
export function ApplicationsPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedTermId, setSelectedTermId] = useState<number | null>(null);
  const [memberId, setMemberId] = useState<number | null>(null);
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
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });

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

  return (
    <div className={styles.page}>
      <TermBar
        terms={terms.data.items}
        current={current}
        onSelect={setSelectedTermId}
        notice={null}
        onNotice={() => undefined}
        canEdit={false}
      />
      <div className={styles.content}>
        <div className={styles.memberSide}>
          <MemberList selectedId={memberId} onSelect={setMemberId} />
          {memberId === null ? (
            <p className={styles.hint}>
              신청을 받을 회원을 목록에서 골라 주세요.
            </p>
          ) : courses.data ? (
            <MemberApplications
              termId={current.id}
              memberId={memberId}
              courses={courses.data.items}
            />
          ) : null}
        </div>
        {courses.isError ? (
          <p className={formStyles.formError} role="alert">
            {courses.error.message}
          </p>
        ) : courses.data && counts.data ? (
          <CourseChoiceList
            courses={courses.data.items}
            counts={countsByCourse(counts.data.items)}
            showBuilding={showBuilding}
          />
        ) : (
          <p className={formStyles.loading}>강좌를 불러오는 중입니다…</p>
        )}
      </div>
    </div>
  );
}
