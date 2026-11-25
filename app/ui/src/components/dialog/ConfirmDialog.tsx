import { useEffect, useId, useRef, type ReactNode } from "react";
import styles from "./ConfirmDialog.module.css";

/** {@link ConfirmDialog}의 속성. */
export interface ConfirmDialogProps {
  /** 묻는 말. 예: "요가 초급을 다시 추첨할까요?" */
  title: string;
  /** 경고 문장. 예: "지우면 되돌릴 수 없습니다." */
  warning: string;
  /** 하기 버튼의 글자. 예: "지우기" */
  confirmLabel: string;
  /** 하는 동안 버튼의 글자. 예: "지우는 중…" */
  busyLabel: string;
  busy: boolean;
  /** 경고 문장 위에 더 보여 줄 것. 예: 사유 칸 */
  children?: ReactNode;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * 되돌리기 어려운 일을 하기 전에 한 번 묻는다. 화면에 뜨는 동안 뒤의 칸은 누를 수 없다.
 *
 * Esc를 누르면 그만둔다. 실수로 하지 않도록 처음 커서는 "그만두기"에 둔다.
 */
export function ConfirmDialog({
  title,
  warning,
  confirmLabel,
  busyLabel,
  busy,
  children,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  return (
    <dialog
      ref={dialog}
      className={styles.dialog}
      aria-labelledby={titleId}
      onCancel={(event) => {
        // Esc: 브라우저가 닫기 전에 상태로 닫음
        event.preventDefault();
        if (!busy) {
          onCancel();
        }
      }}
    >
      <h2 id={titleId} className={styles.title}>
        {title}
      </h2>
      {children}
      <p className={styles.body}>{warning}</p>
      <div className={styles.actions}>
        <button
          type="button"
          className={styles.cancel}
          onClick={onCancel}
          disabled={busy}
          autoFocus
        >
          그만두기
        </button>
        <button
          type="button"
          className={styles.confirm}
          onClick={onConfirm}
          disabled={busy}
        >
          {busy ? busyLabel : confirmLabel}
        </button>
      </div>
    </dialog>
  );
}
