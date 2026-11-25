import { useEffect, useId, useRef, useState } from "react";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { RuleViolationResponse } from "../../api/generated/RuleViolationResponse";
import formStyles from "../../components/form/form.module.css";
import { violationMessage } from "./applicationsData";
import styles from "./RuleViolationDialog.module.css";

/** {@link RuleViolationDialog}의 속성. */
export interface RuleViolationDialogProps {
  violations: readonly RuleViolationResponse[];
  /** 이 학기의 강좌. 강좌 이름을 찾는다. */
  courses: readonly CourseResponse[];
  /** 규칙 예외를 줄 수 있는 역할인지 (BR-K3). */
  canOverride: boolean;
  saving: boolean;
  /** 호스트가 사유를 받지 않았을 때의 문장. 예: 너무 김 */
  reasonError: string | null;
  /** 사유를 적고 다시 신청한다(BR-A6). */
  onOverride: (reason: string) => void;
  onCancel: () => void;
}

/**
 * 신청 규칙을 어겼을 때 어긴 규칙을 모두 보여 준다(BR-A5).
 *
 * 모두 예외를 줄 수 있는 규칙이고 역할이 되면 사유를 받아 그대로 신청한다(BR-A6).
 * 같은 강좌 두 번(BR-A4)이 끼어 있으면 예외 없이 강좌를 빼라고 안내한다(BR-A7).
 */
export function RuleViolationDialog({
  violations,
  courses,
  canOverride,
  saving,
  reasonError,
  onOverride,
  onCancel,
}: RuleViolationDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const reasonId = useId();
  const [reason, setReason] = useState("");

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  const overridable = violations.every((violation) => violation.canOverride);

  return (
    <dialog
      ref={dialog}
      className={styles.dialog}
      aria-labelledby={titleId}
      onCancel={(event) => {
        // Esc: 브라우저가 닫기 전에 상태로 닫음
        event.preventDefault();
        if (!saving) {
          onCancel();
        }
      }}
    >
      <h2 id={titleId} className={styles.title}>
        신청 규칙을 어긴 강좌가 있습니다
      </h2>
      <ul className={styles.violations}>
        {violations.map((violation) => (
          <li
            key={`${violation.courseId}-${violation.rule}-${violation.otherCourseId}`}
          >
            {violationMessage(violation, courses)}
          </li>
        ))}
      </ul>

      {!overridable ? (
        <p className={styles.body}>
          이미 신청한 강좌는 다시 신청할 수 없습니다. 그 강좌를 빼고 다시 신청해
          주세요.
        </p>
      ) : !canOverride ? (
        <p className={styles.body}>
          강좌를 빼고 다시 신청해 주세요. 사정이 있으면 직원이나 관리자가 사유를
          적고 받을 수 있습니다.
        </p>
      ) : (
        <form
          id={`${titleId}-form`}
          className={styles.reason}
          onSubmit={(event) => {
            event.preventDefault();
            onOverride(reason);
          }}
        >
          <label htmlFor={reasonId} className={formStyles.label}>
            사유를 적으면 그대로 신청합니다
          </label>
          <input
            id={reasonId}
            className={formStyles.input}
            value={reason}
            onChange={(event) => setReason(event.target.value)}
            placeholder="예: 강사 허락"
            maxLength={100}
            aria-invalid={reasonError ? true : undefined}
            autoFocus
          />
          {reasonError && (
            <p className={formStyles.fieldError} role="alert">
              {reasonError}
            </p>
          )}
        </form>
      )}

      <div className={styles.actions}>
        <button
          type="button"
          className={formStyles.secondary}
          onClick={onCancel}
          disabled={saving}
        >
          강좌 다시 고르기
        </button>
        {overridable && canOverride && (
          <button
            type="submit"
            form={`${titleId}-form`}
            className={formStyles.submit}
            disabled={saving || reason.trim() === ""}
          >
            {saving ? "신청하는 중…" : "사유 적고 신청"}
          </button>
        )}
      </div>
    </dialog>
  );
}
