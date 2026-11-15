import { useEffect, useRef } from "react";
import styles from "./DeleteMemberDialog.module.css";

/** {@link DeleteMemberDialog}의 속성. */
export interface DeleteMemberDialogProps {
  name: string;
  deleting: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * 회원을 지우기 전에 한 번 묻는다. 화면에 뜨는 동안 뒤의 칸은 누를 수 없다.
 *
 * Esc를 누르면 그만둔다. 실수로 지우지 않도록 처음 커서는 "그만두기"에 둔다.
 */
export function DeleteMemberDialog({
  name,
  deleting,
  onConfirm,
  onCancel,
}: DeleteMemberDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  return (
    <dialog
      ref={dialog}
      className={styles.dialog}
      aria-labelledby="delete-member-title"
      onCancel={(event) => {
        // Esc: 브라우저가 닫기 전에 상태로 닫음
        event.preventDefault();
        if (!deleting) {
          onCancel();
        }
      }}
    >
      <h2 id="delete-member-title" className={styles.title}>
        {name}님을 지울까요?
      </h2>
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
