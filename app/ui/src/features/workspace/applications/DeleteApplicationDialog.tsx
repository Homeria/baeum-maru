import { useId, useRef, useState } from "react";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import styles from "./ApplicationsPage.module.css";

/** {@link DeleteApplicationDialog}의 속성. */
export interface DeleteApplicationDialogProps {
  /** 지울 신청의 강좌 이름. 예: 요가 초급 */
  courseName: string;
  deleting: boolean;
  /** 호스트가 사유를 받지 않았을 때의 문장. 예: 너무 김 */
  reasonError: string | null;
  /** 사유는 비었으면 `null`. */
  onConfirm: (reason: string | null) => void;
  onCancel: () => void;
}

/** 신청을 지우기 전에 묻고 사유를 받는 모달 대화상자(선택). 호스트가 삭제 기록을 남긴다(BR-D3). */
export function DeleteApplicationDialog({
  courseName,
  deleting,
  reasonError,
  onConfirm,
  onCancel,
}: DeleteApplicationDialogProps) {
  const formId = useId();
  const [reason, setReason] = useState("");
  const cancelRef = useRef<HTMLButtonElement>(null);

  return (
    <ModalDialog
      open
      size="sm"
      title={`${courseName} 신청을 지울까요?`}
      busy={deleting}
      initialFocusRef={cancelRef}
      onClose={onCancel}
      footer={
        <>
          <Button
            ref={cancelRef}
            variant="secondary"
            disabled={deleting}
            onClick={onCancel}
          >
            그만두기
          </Button>
          <Button
            type="submit"
            form={formId}
            variant="dangerSolid"
            busy={deleting}
            busyLabel="지우는 중…"
          >
            지우기
          </Button>
        </>
      }
    >
      <form
        id={formId}
        className={styles.dialogBody}
        onSubmit={(event) => {
          event.preventDefault();
          if (!deleting) {
            onConfirm(reason.trim() === "" ? null : reason);
          }
        }}
        noValidate
      >
        <TextField
          label="사유 (선택)"
          placeholder="예: 잘못 고름, 회원이 마음을 바꿈"
          maxLength={100}
          value={reason}
          onChange={(event) => setReason(event.target.value)}
          error={reasonError ?? undefined}
        />
      </form>
    </ModalDialog>
  );
}
