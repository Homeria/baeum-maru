import {
  useEffect,
  useId,
  useRef,
  useState,
  type PointerEvent,
  type ReactNode,
} from "react";
import { Button } from "../ui/Button";
import { ModalDialog } from "./ModalDialog";
import styles from "./Drawer.module.css";

/** {@link Drawer}의 속성. */
export interface DrawerProps {
  /** 서랍 열림 여부. */
  open: boolean;
  /** 서랍 머리줄 제목. `aria-labelledby`로 연결된다. */
  title: string;
  /** 제목 아래 짧은 보조 설명. */
  subtitle?: string;
  /** 편집 중인 바뀐 내용이 있는지 여부. 참이면 닫기 전에 한 번 더 묻는다. */
  isDirty?: boolean;
  /** 저장·삭제 처리 중 여부. 참이면 닫기를 막는다. */
  busy?: boolean;
  /** 서랍을 닫을 때 호출된다. */
  onClose: () => void;
  /** 바닥 왼쪽 영역(예: 지우기 버튼). */
  footerLeft?: ReactNode;
  /** 바닥 오른쪽 영역(예: 그만두기, 저장 버튼). */
  footerRight?: ReactNode;
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
 * 화면 오른쪽에서 열리는 모달 편집 서랍.
 *
 * 바꾼 내용(`isDirty`)이 없으면 바깥 클릭·Esc·닫기 버튼으로 바로 닫히고,
 * 바꾼 내용이 있으면 실수로 잃지 않도록 닫기 확인 대화상자를 띄운다.
 */
export function Drawer({
  open,
  title,
  subtitle,
  isDirty = false,
  busy = false,
  onClose,
  footerLeft,
  footerRight,
  children,
  className,
}: DrawerProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const keepEditingRef = useRef<HTMLButtonElement>(null);
  const pressedOutsideRef = useRef(false);
  const [confirmingClose, setConfirmingClose] = useState(false);
  const titleId = useId();

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) {
      return;
    }
    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  if (!open) {
    return null;
  }

  function requestClose() {
    if (busy) {
      return;
    }
    if (isDirty) {
      setConfirmingClose(true);
      return;
    }
    onClose();
  }

  function handlePointerDown(event: PointerEvent<HTMLDialogElement>) {
    const dialog = dialogRef.current;
    if (!dialog || busy || confirmingClose || event.target !== dialog) {
      pressedOutsideRef.current = false;
      return;
    }
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
      busy ||
      confirmingClose ||
      !wasOutside ||
      event.target !== dialog
    ) {
      return;
    }
    if (isOutsideRect(dialog, event.clientX, event.clientY)) {
      requestClose();
    }
  }

  // 닫기 확인 가드가 일관되게 돌도록 바깥 클릭은 pointerdown/pointerup으로 제어한다
  const dialogAttrs: Record<string, string> = {
    closedby: busy || isDirty ? "none" : "closerequest",
  };
  const hasFooter = Boolean(footerLeft || footerRight);

  return (
    <dialog
      {...dialogAttrs}
      ref={dialogRef}
      aria-labelledby={titleId}
      className={[styles.drawer, className].filter(Boolean).join(" ")}
      onCancel={(event) => {
        event.preventDefault();
        requestClose();
      }}
      onPointerDown={handlePointerDown}
      onPointerUp={handlePointerUp}
    >
      <div className={styles.header}>
        <div className={styles.headingGroup}>
          <h1 id={titleId} className={styles.title}>
            {title}
          </h1>
          {subtitle && <p className={styles.subtitle}>{subtitle}</p>}
        </div>
        <Button
          size="sm"
          variant="secondary"
          disabled={busy}
          onClick={requestClose}
        >
          닫기
        </Button>
      </div>

      <div className={styles.body}>{children}</div>

      {hasFooter && (
        <div className={styles.footer}>
          <div className={styles.footerLeft}>{footerLeft}</div>
          <div className={styles.footerRight}>{footerRight}</div>
        </div>
      )}

      <ModalDialog
        open={confirmingClose}
        size="sm"
        title="저장하지 않은 내용이 있습니다"
        initialFocusRef={keepEditingRef}
        onClose={() => setConfirmingClose(false)}
        footer={
          <>
            <Button
              ref={keepEditingRef}
              onClick={() => setConfirmingClose(false)}
            >
              계속 편집
            </Button>
            <Button
              variant="dangerSolid"
              onClick={() => {
                setConfirmingClose(false);
                onClose();
              }}
            >
              버리고 닫기
            </Button>
          </>
        }
      >
        <p className={styles.confirmText}>
          지금 닫으면 고친 내용이 사라집니다. 그대로 닫을까요?
        </p>
      </ModalDialog>
    </dialog>
  );
}
