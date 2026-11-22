import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { deleteApplication, listApplications } from "../../api/applications";
import { HostError } from "../../api/client";
import type { ApplicationResponse } from "../../api/generated/ApplicationResponse";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import { getMember } from "../../api/members";
import formStyles from "../../components/form/form.module.css";
import tableStyles from "../../components/table/table.module.css";
import { useCan } from "../auth/useCan";
import { courseName, scheduleSummary } from "../courses/courseFormData";
import { RULE_LABELS } from "./applicationsData";
import { DeleteApplicationDialog } from "./DeleteApplicationDialog";
import styles from "./ApplicationsPage.module.css";

/** {@link MemberApplications}의 속성. */
export interface MemberApplicationsProps {
  termId: number;
  memberId: number;
  /** 이 학기의 강좌. 신청한 강좌의 이름과 시간을 찾는다. */
  courses: readonly CourseResponse[];
}

/**
 * 고른 회원이 이 학기에 낸 신청. 접수한 순서이고, 규칙 예외를 받은 신청은 표시한다.
 *
 * 신청을 받을 수 있는 역할(BR-K3)은 신청마다 지우기 버튼이 있다(BR-D1~D3).
 */
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
  const queryClient = useQueryClient();
  const canDelete = useCan("takeApplications");
  const [deleting, setDeleting] = useState<ApplicationResponse | null>(null);
  const [reasonError, setReasonError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const nameOf = (courseId: number) => {
    const course = courses.find((candidate) => candidate.id === courseId);
    return course ? courseName(course) : "지워진 강좌";
  };

  const remove = useMutation({
    mutationFn: ({ id, reason }: { id: number; reason: string | null }) =>
      deleteApplication(id, { reason }),
    onSuccess: () => {
      setNotice(`지웠습니다: ${deleting ? nameOf(deleting.courseId) : ""}`);
      setDeleting(null);
      setReasonError(null);
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
      void queryClient.invalidateQueries({
        queryKey: ["applicationCounts", termId],
      });
    },
    onError: (failure) => {
      if (failure instanceof HostError && failure.fields.length > 0) {
        // 사유가 받아들여지지 않음 (예: 너무 김). 창을 열어 둔다
        setReasonError(failure.fields[0].message);
        return;
      }
      setDeleting(null);
      setError(failure.message);
      // 그 사이 지워졌거나 선발됐을 수 있어 새로 받음
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
    },
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
      {notice && (
        <p className={formStyles.notice} role="status">
          {notice}
        </p>
      )}
      {error && (
        <p className={formStyles.formError} role="alert">
          {error}
        </p>
      )}
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
                  {nameOf(application.courseId)}
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
                {canDelete && (
                  <button
                    type="button"
                    className={`${formStyles.danger} ${styles.deleteButton}`}
                    onClick={() => {
                      setNotice(null);
                      setError(null);
                      setReasonError(null);
                      setDeleting(application);
                    }}
                  >
                    지우기
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}

      {deleting && (
        <DeleteApplicationDialog
          courseName={nameOf(deleting.courseId)}
          deleting={remove.isPending}
          reasonError={reasonError}
          onConfirm={(reason) => remove.mutate({ id: deleting.id, reason })}
          onCancel={() => setDeleting(null)}
        />
      )}
    </section>
  );
}
