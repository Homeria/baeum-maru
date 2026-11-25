import { useEffect, useRef } from "react";
import { TermForm, type TermFormProps } from "./TermForm";
import styles from "./TermDialog.module.css";

/** {@link TermDialog}의 속성. 그만두기가 늘 있다. */
export type TermDialogProps = TermFormProps & { onCancel: () => void };

/**
 * 학기 입력 칸을 작은 창으로 띄운다. 창이 떠 있는 동안 뒤의 강좌 목록은 누를 수 없다.
 *
 * Esc를 누르면 그만둔다.
 */
export function TermDialog(props: TermDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  return (
    <dialog
      ref={dialog}
      className={styles.dialog}
      aria-label={props.editing ? "학기 고치기" : "새 학기 등록"}
      onCancel={(event) => {
        // Esc: 브라우저가 닫기 전에 상태로 닫음
        event.preventDefault();
        // 안쪽 지우기 확인 창의 Esc는 그 창만 닫음 (React 이벤트는 부모 부품으로 전달됨)
        if (event.target === event.currentTarget) {
          props.onCancel();
        }
      }}
    >
      <TermForm {...props} />
    </dialog>
  );
}
