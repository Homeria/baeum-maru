import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { submitApplications } from "../../api/applications";
import { HostError } from "../../api/client";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { RuleViolationResponse } from "../../api/generated/RuleViolationResponse";
import formStyles from "../../components/form/form.module.css";
import { useCan } from "../auth/useCan";
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
  /** 신청을 저장했다. 고른 강좌를 비운다. */
  onSubmitted: () => void;
}

/**
 * 강좌 목록 아래의 신청 줄. 고른 강좌를 한 번에 신청한다(접수).
 *
 * 규칙을 어기면 창을 띄워 어긴 규칙을 보여 주고, 사유를 받으면 그대로 다시 신청한다.
 */
export function ApplyBar({
  termId,
  memberId,
  chosen,
  courses,
  onSubmitted,
}: ApplyBarProps) {
  const queryClient = useQueryClient();
  const canOverride = useCan("overrideRules");
  const [violations, setViolations] = useState<
    readonly RuleViolationResponse[] | null
  >(null);
  const [reasonError, setReasonError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const submit = useMutation({
    mutationFn: (overrideReason: string | null) =>
      submitApplications(termId, {
        // 버튼이 회원을 고른 뒤에만 눌리므로 0으로 보낼 일은 없음
        memberId: memberId ?? 0,
        courseIds: [...chosen],
        overrideReason,
      }),
    onSuccess: (response) => {
      setViolations(null);
      setReasonError(null);
      setError(null);
      setNotice(`${response.ids.length}건 신청했습니다`);
      onSubmitted();
      void queryClient.invalidateQueries({
        queryKey: ["applications", termId, memberId],
      });
      void queryClient.invalidateQueries({
        queryKey: ["applicationCounts", termId],
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

  return (
    <div className={styles.applyBar}>
      {error && (
        <p className={formStyles.formError} role="alert">
          {error}
        </p>
      )}
      {notice && count === 0 && (
        <p className={formStyles.notice} role="status">
          {notice}
        </p>
      )}
      <button
        type="button"
        className={formStyles.submit}
        disabled={memberId === null || count === 0 || submit.isPending}
        onClick={() => {
          setNotice(null);
          setError(null);
          submit.mutate(null);
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
          onOverride={(reason) => submit.mutate(reason)}
          onCancel={() => setViolations(null)}
        />
      )}
    </div>
  );
}
