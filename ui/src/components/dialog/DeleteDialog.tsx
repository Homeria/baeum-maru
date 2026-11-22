import { useEffect, useId, useRef, type ReactNode } from "react";
import styles from "./DeleteDialog.module.css";

/** {@link DeleteDialog}의 속성. */
export interface DeleteDialogProps {
  /** 묻는 말. 예: "회원1님을 지울까요?" */
  title: string;
  deleting: boolean;
  /** 경고 문장 위에 더 보여 줄 것. 예: 지우는 사유 칸 */
  children?: ReactNode;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * 지우기 전에 한 번 묻는다. 화면에 뜨는 동안 뒤의 칸은 누를 수 없다.
 *
 * Esc를 누르면 그만둔다. 실수로 지우지 않도록 처음 커서는 "그만두기"에 둔다.
 */
export function DeleteDialog({
  title,
  deleting,
  children,
  onConfirm,
  onCancel,
}: DeleteDialogProps) {
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
        if (!deleting) {
          onCancel();
        }
      }}
    >
      <h2 id={titleId} className={styles.title}>
        {title}
      </h2>
      {children}
      <p className={styles.body}>지우면 되돌릴 수 없습니다.</p>
      <div className={styles.actions}>
        <button
          type="button"
          className={styles.cancel}
          onClick={onCancel}
          disabled={deleting}
          autoFocus
        >
          그만두기
        </button>
        <button
          type="button"
          className={styles.confirm}
          onClick={onConfirm}
          disabled={deleting}
        >
          {deleting ? "지우는 중…" : "지우기"}
        </button>
      </div>
    </dialog>
  );
}
