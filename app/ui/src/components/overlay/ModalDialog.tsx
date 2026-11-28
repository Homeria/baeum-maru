import { useId, type ReactNode, type RefObject } from "react";
import { useModalDialog } from "./useModalDialog";
import styles from "./ModalDialog.module.css";

/** {@link ModalDialog}의 속성. */
export interface ModalDialogProps {
  /** 대화상자 열림 여부. */
  open: boolean;
  /** 대화상자 제목. `aria-labelledby`로 연결된다. */
  title: string;
  /** 닫기 요청 시 호출된다(Esc, 닫기 버튼, 바깥 클릭). */
  onClose: () => void;
  /** 작업 처리 중 여부. 참이면 Esc·바깥 클릭으로 닫히지 않는다. */
  busy?: boolean;
  /** 바깥 어두운 영역을 눌러 닫을 수 있는지 여부. 기본값은 `false`. */
  lightDismiss?: boolean;
  /** 가로 폭. 확인 창은 `"sm"`, 입력 창은 `"md"`. */
  size?: "sm" | "md";
  /** 열릴 때 처음 초점을 둘 요소 참조(예: 안전한 "그만두기" 버튼). */
  initialFocusRef?: RefObject<HTMLElement | null>;
  /** 바닥 오른쪽에 배치할 버튼 영역. */
  footer?: ReactNode;
  children: ReactNode;
  className?: string;
}

/**
 * 네이티브 `<dialog>`의 `showModal()`로 띄우는 공통 모달 대화상자.
 *
 * 열릴 때 뒤쪽 화면 비활성(`inert`)과 포커스 가둠을 제공하고, 닫히거나 언마운트될 때 원래 버튼으로 초점을 돌려준다.
 */
export function ModalDialog({
  open,
  title,
  onClose,
  busy = false,
  lightDismiss = false,
  size = "md",
  initialFocusRef,
  footer,
  children,
  className,
}: ModalDialogProps) {
  const titleId = useId();
  const {
    dialogRef,
    isClosingProgrammatically,
    handlePointerDown,
    handlePointerUp,
  } = useModalDialog({
    open,
    allowOutsideDismiss: lightDismiss && !busy,
    deferToNativeClosedBy: true,
    initialFocusRef,
    onOutsideDismiss: onClose,
  });

  if (!open) {
    return null;
  }

  const closedby = busy ? "none" : lightDismiss ? "any" : "closerequest";
  const dialogAttrs: Record<string, string> = { closedby };

  return (
    <dialog
      {...dialogAttrs}
      ref={dialogRef}
      aria-labelledby={titleId}
      className={[styles.dialog, styles[size], className]
        .filter(Boolean)
        .join(" ")}
      onCancel={(event) => {
        event.preventDefault();
        if (!busy) {
          onClose();
        }
      }}
      onClose={() => {
        if (dialogRef.current) {
          dialogRef.current.returnValue = "";
        }
        if (open && !busy && !isClosingProgrammatically()) {
          onClose();
        }
      }}
      onPointerDown={handlePointerDown}
      onPointerUp={handlePointerUp}
    >
      <div className={styles.header}>
        <h1 id={titleId} className={styles.title}>
          {title}
        </h1>
      </div>
      <div className={styles.body}>{children}</div>
      {footer && <div className={styles.footer}>{footer}</div>}
    </dialog>
  );
}
