import { useId, useState } from "react";
import { DeleteDialog } from "../../components/dialog/DeleteDialog";
import formStyles from "../../components/form/form.module.css";
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

/** 신청을 지우기 전에 묻고 사유를 받는다(선택). 호스트가 삭제 기록을 남긴다(BR-D3). */
export function DeleteApplicationDialog({
  courseName,
  deleting,
  reasonError,
  onConfirm,
  onCancel,
}: DeleteApplicationDialogProps) {
  const reasonId = useId();
  const [reason, setReason] = useState("");

  return (
    <DeleteDialog
      title={`${courseName} 신청을 지울까요?`}
      deleting={deleting}
      onConfirm={() => onConfirm(reason.trim() === "" ? null : reason)}
      onCancel={onCancel}
    >
      <div className={styles.deleteReason}>
        <label htmlFor={reasonId} className={formStyles.label}>
          사유 (선택)
        </label>
        <input
          id={reasonId}
          className={formStyles.input}
          value={reason}
          onChange={(event) => setReason(event.target.value)}
          placeholder="예: 잘못 고름, 회원이 마음을 바꿈"
          maxLength={100}
          aria-invalid={reasonError ? true : undefined}
        />
        {reasonError && (
          <p className={formStyles.fieldError} role="alert">
            {reasonError}
          </p>
        )}
      </div>
    </DeleteDialog>
  );
}
