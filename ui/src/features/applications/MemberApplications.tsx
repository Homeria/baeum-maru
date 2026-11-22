import { useQuery } from "@tanstack/react-query";
import { listApplications } from "../../api/applications";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import { getMember } from "../../api/members";
import tableStyles from "../../components/table/table.module.css";
import { courseName, scheduleSummary } from "../courses/courseFormData";
import { RULE_LABELS } from "./applicationsData";
import styles from "./ApplicationsPage.module.css";

/** {@link MemberApplications}의 속성. */
export interface MemberApplicationsProps {
  termId: number;
  memberId: number;
  /** 이 학기의 강좌. 신청한 강좌의 이름과 시간을 찾는다. */
  courses: readonly CourseResponse[];
}

/** 고른 회원이 이 학기에 낸 신청. 접수한 순서이고, 규칙 예외를 받은 신청은 표시한다. */
export function MemberApplications({
  termId,
  memberId,
  courses,
}: MemberApplicationsProps) {
  const member = useQuery({
    queryKey: ["members", "detail", memberId],
    queryFn: () => getMember(memberId),
  });
  const applications = useQuery({
    queryKey: ["applications", termId, memberId],
    queryFn: () => listApplications(termId, memberId),
  });

  if (applications.isPending) {
    return <p className={tableStyles.message}>신청을 불러오는 중입니다…</p>;
  }
  if (applications.isError) {
    return (
      <p className={tableStyles.error} role="alert">
        {applications.error.message}
      </p>
    );
  }

  const items = applications.data.items;
  const title = member.data
    ? `${member.data.name}(${member.data.memberNo})의 신청`
    : "고른 회원의 신청";

  return (
    <section className={styles.applications} aria-label="고른 회원의 신청">
      <h2 className={styles.heading}>
        {title} · {items.length}건
      </h2>
      {items.length === 0 ? (
        <p className={tableStyles.message}>이 학기에 낸 신청이 없습니다</p>
      ) : (
        <ul className={styles.applicationList}>
          {items.map((application) => {
            const course = courses.find(
              (candidate) => candidate.id === application.courseId,
            );
            return (
              <li key={application.id} className={styles.application}>
                <span className={styles.courseName}>
                  {course ? courseName(course) : "지워진 강좌"}
                </span>
                {course && (
                  <span className={styles.muted}>
                    {scheduleSummary(course.schedules, false).times}
                  </span>
                )}
                {application.overriddenRules.length > 0 && (
                  <span className={styles.override}>
                    예외:{" "}
                    {application.overriddenRules
                      .map((rule) => RULE_LABELS[rule])
                      .join(", ")}
                  </span>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
