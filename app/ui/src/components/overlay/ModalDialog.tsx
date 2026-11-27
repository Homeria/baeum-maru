import {
  useEffect,
  useId,
  useRef,
  type PointerEvent,
  type ReactNode,
  type RefObject,
} from "react";
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

function isOutsideRect(
  dialog: HTMLDialogElement,
  clientX: number,
  clientY: number,
): boolean {
  const rect = dialog.getBoundingClientRect();
  return (
    clientX < rect.left ||
    clientX > rect.right ||
    clientY < rect.top ||
    clientY > rect.bottom
  );
}

/**
 * 네이티브 `<dialog>`의 `showModal()`로 띄우는 공통 모달 대화상자.
 *
 * 브라우저가 뒤쪽 화면 비활성(`inert`), 포커스 가둠, 닫힌 뒤 원래 버튼으로 포커스 복귀를 처리한다.
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
  const dialogRef = useRef<HTMLDialogElement>(null);
  const pressedOutsideRef = useRef(false);
  const titleId = useId();

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) {
      return;
    }
    if (open && !dialog.open) {
      dialog.showModal();
      initialFocusRef?.current?.focus();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open, initialFocusRef]);

  if (!open) {
    return null;
  }

  const closedby = busy ? "none" : lightDismiss ? "any" : "closerequest";
  const dialogAttrs: Record<string, string> = { closedby };

  function handlePointerDown(event: PointerEvent<HTMLDialogElement>) {
    const dialog = dialogRef.current;
    if (!dialog || !lightDismiss || busy || event.target !== dialog) {
      pressedOutsideRef.current = false;
      return;
    }
    // 텍스트 드래그가 바깥에서 끝나 닫히는 실수를 막으려고 눌렀을 때와 뗐을 때 모두 바깥인지 본다
    pressedOutsideRef.current = isOutsideRect(
      dialog,
      event.clientX,
      event.clientY,
    );
  }

  function handlePointerUp(event: PointerEvent<HTMLDialogElement>) {
    const dialog = dialogRef.current;
    const wasOutside = pressedOutsideRef.current;
    pressedOutsideRef.current = false;
    if (
      !dialog ||
      !lightDismiss ||
      busy ||
      !wasOutside ||
      event.target !== dialog
    ) {
      return;
    }
    if ("closedBy" in HTMLDialogElement.prototype) {
      return;
    }
    if (isOutsideRect(dialog, event.clientX, event.clientY)) {
      onClose();
    }
  }

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
        if (open && !busy) {
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
