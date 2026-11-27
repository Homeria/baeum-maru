import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { submitApplications } from "../../api/applications";
import { HostError } from "../../api/client";
import type { AdditionalOutcomeResponse } from "../../api/generated/AdditionalOutcomeResponse";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { PriorityReasonResponse } from "../../api/generated/PriorityReasonResponse";
import type { RuleViolationResponse } from "../../api/generated/RuleViolationResponse";
import formStyles from "../../components/form/form.module.css";
import { useCan } from "../gate/auth/useCan";
import { courseName } from "../courses/courseFormData";
import {
  priorityRequests,
  submittedNotice,
  type ChosenPriorities,
} from "./applicationsData";
import { PriorityChoices } from "./PriorityChoices";
import { RuleViolationDialog } from "./RuleViolationDialog";
import styles from "./ApplicationsPage.module.css";

/** {@link ApplyBar}의 속성. */
export interface ApplyBarProps {
  termId: number;
  /** 고른 회원. 없으면 신청할 수 없다. */
  memberId: number | null;
  /** 고른 강좌. 고른 순서. */
  chosen: readonly number[];
  courses: readonly CourseResponse[];
  /** 우선 배정 사유. 없으면 우선 배정 칸을 보여 주지 않는다. */
  reasons: readonly PriorityReasonResponse[];
  /** 고른 강좌에 붙일 사유. */
  priorities: ChosenPriorities;
  onTogglePriority: (courseId: number, reasonId: number) => void;
  /** 마감한 강좌의 추가 신청 미리 보기(BR-X2). 결과가 달라졌는지 알리는 데 쓴다. */
  additional: ReadonlyMap<number, AdditionalOutcomeResponse>;
  /** 신청을 저장했다. 고른 강좌와 사유를 비운다. */
  onSubmitted: () => void;
}

/**
 * 강좌 목록 아래의 신청 줄. 고른 강좌를 한 번에 신청한다(접수).
 *
 * 규칙을 어기면 창을 띄워 어긴 규칙을 보여 주고, 사유를 받으면 그대로 다시 신청한다.
 * 사유 목록이 있고 우선 배정을 할 수 있는 역할이면, 고른 강좌마다 사유 체크박스를 붙인다(BR-S12).
 * 추가 신청으로 고른 마감 강좌에는 붙이지 않는다(BR-S13).
 */
export function ApplyBar({
  termId,
  memberId,
  chosen,
  courses,
  reasons,
  priorities,
  onTogglePriority,
  additional,
  onSubmitted,
}: ApplyBarProps) {
  const queryClient = useQueryClient();
  const canOverride = useCan("overrideRules");
  const canAssign = useCan("assignPriority");
  const [violations, setViolations] = useState<
    readonly RuleViolationResponse[] | null
  >(null);
  const [reasonError, setReasonError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<readonly string[] | null>(null);
  // 추가 신청(마감한 강좌)에는 우선 배정 사유를 붙이지 않는다 (BR-S13)
  const priorityCourseIds = chosen.filter((id) => !additional.has(id));

  const submit = useMutation({
    // previews: 누른 때 보였던 미리 보기. 응답이 오기 전에 화면이 새로 받아도 비교할 수 있게 함께 넘김
    mutationFn: ({
      overrideReason,
    }: {
      overrideReason: string | null;
      previews: ReadonlyMap<number, AdditionalOutcomeResponse>;
    }) =>
      submitApplications(termId, {
        // 버튼이 회원을 고른 뒤에만 눌리므로 0으로 보낼 일은 없음
        memberId: memberId ?? 0,
        courseIds: [...chosen],
        overrideReason,
        priorities: canAssign
          ? priorityRequests(priorityCourseIds, priorities)
          : [],
      }),
    onSuccess: (response, { previews }) => {
      setViolations(null);
      setReasonError(null);
      setError(null);
      setNotice(submittedNotice(response.items, previews, courses));
      onSubmitted();
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
      void queryClient.invalidateQueries({
        queryKey: ["applicationCounts", termId],
      });
      // 추가 신청이 빈자리와 다음 대기 순번을 바꾼다 (BR-X2)
      void queryClient.invalidateQueries({
        queryKey: ["closed-courses", termId],
      });
    },
    onError: (failure) => {
      if (failure instanceof HostError && failure.code === "RULES_BROKEN") {
        setViolations(failure.violations);
        setReasonError(null);
      } else if (
        violations !== null &&
        failure instanceof HostError &&
        failure.fields.length > 0
      ) {
        // 창에서 보낸 사유가 받아들여지지 않음 (예: 너무 김)
        setReasonError(failure.fields[0].message);
      } else {
        setViolations(null);
        setError(failure.message);
      }
    },
  });

  const count = chosen.length;
  const showPriorities =
    canAssign && reasons.length > 0 && priorityCourseIds.length > 0;

  return (
    <>
      {showPriorities && (
        <ul
          className={styles.chosenPriorities}
          aria-label="고른 강좌의 우선 배정"
        >
          {priorityCourseIds.map((courseId) => {
            const course = courses.find(
              (candidate) => candidate.id === courseId,
            );
            const name = course ? courseName(course) : "지워진 강좌";
            return (
              <li key={courseId} className={styles.chosenPriority}>
                <span className={styles.courseName}>{name}</span>
                <PriorityChoices
                  reasons={reasons}
                  checked={priorities.get(courseId) ?? []}
                  courseName={name}
                  disabled={submit.isPending}
                  onToggle={(reasonId) => onTogglePriority(courseId, reasonId)}
                />
              </li>
            );
          })}
        </ul>
      )}
      <div className={styles.applyBar}>
        {error && (
          <p className={formStyles.formError} role="alert">
            {error}
          </p>
        )}
        {notice && count === 0 && (
          <div className={formStyles.notice} role="status">
            {notice.map((line) => (
              <p key={line} className={styles.noticeLine}>
                {line}
              </p>
            ))}
          </div>
        )}
        <button
          type="button"
          className={formStyles.submit}
          disabled={memberId === null || count === 0 || submit.isPending}
          onClick={() => {
            setNotice(null);
            setError(null);
            submit.mutate({ overrideReason: null, previews: additional });
          }}
        >
          {submit.isPending && violations === null
            ? "신청하는 중…"
            : `${count}건 신청하기`}
        </button>

        {violations && (
          <RuleViolationDialog
            violations={violations}
            courses={courses}
            canOverride={canOverride}
            saving={submit.isPending}
            reasonError={reasonError}
            onOverride={(reason) =>
              submit.mutate({ overrideReason: reason, previews: additional })
            }
            onCancel={() => setViolations(null)}
          />
        )}
      </div>
    </>
  );
}
