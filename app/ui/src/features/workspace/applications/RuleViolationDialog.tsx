import { useId, useRef, useState } from "react";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { RuleViolationResponse } from "../../../api/generated/RuleViolationResponse";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import { violationMessage } from "./applicationsData";
import styles from "./ApplicationsPage.module.css";

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
 * 신청 규칙을 어겼을 때 어긴 규칙을 모두 보여 주는 모달 대화상자(BR-A5).
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
  const formId = useId();
  const [reason, setReason] = useState("");
  const cancelRef = useRef<HTMLButtonElement>(null);
  const reasonInputRef = useRef<HTMLInputElement>(null);

  const overridable = violations.every((violation) => violation.canOverride);
  const allowOverride = overridable && canOverride;

  return (
    <ModalDialog
      open
      size="md"
      title="신청 규칙을 어긴 강좌가 있습니다"
      busy={saving}
      initialFocusRef={allowOverride ? reasonInputRef : cancelRef}
      onClose={onCancel}
      footer={
        <>
          <Button
            ref={cancelRef}
            variant="secondary"
            disabled={saving}
            onClick={onCancel}
          >
            강좌 다시 고르기
          </Button>
          {allowOverride && (
            <Button
              type="submit"
              form={formId}
              variant="primary"
              busy={saving}
              busyLabel="신청하는 중…"
              disabled={reason.trim() === ""}
            >
              사유 적고 신청
            </Button>
          )}
        </>
      }
    >
      <div className={styles.dialogBody}>
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
          <NoticeBanner
            tone="error"
            message="이미 신청한 강좌는 다시 신청할 수 없습니다. 그 강좌를 빼고 다시 신청해 주세요."
          />
        ) : !canOverride ? (
          <NoticeBanner message="강좌를 빼고 다시 신청해 주세요. 사정이 있으면 직원이나 관리자가 사유를 적고 받을 수 있습니다." />
        ) : (
          <form
            id={formId}
            onSubmit={(event) => {
              event.preventDefault();
              if (!saving && reason.trim() !== "") {
                onOverride(reason);
              }
            }}
            noValidate
          >
            <TextField
              ref={reasonInputRef}
              label="예외 사유"
              required
              hint="사유를 적으면 그대로 신청합니다."
              placeholder="예: 강사 허락"
              maxLength={100}
              value={reason}
              onChange={(event) => setReason(event.target.value)}
              error={reasonError ?? undefined}
            />
          </form>
        )}
      </div>
    </ModalDialog>
  );
}
