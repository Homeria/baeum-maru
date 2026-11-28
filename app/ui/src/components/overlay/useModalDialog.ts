import { useEffect, useRef, type PointerEvent, type RefObject } from "react";

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

/** {@link useModalDialog}의 선택지. */
export interface UseModalDialogOptions {
  /** 대화상자 열림 여부. */
  open: boolean;
  /** 바깥 어두운 영역 클릭으로 닫을 수 있는지 여부. */
  allowOutsideDismiss: boolean;
  /** 브라우저가 네이티브 `closedBy`를 지원할 때 바깥 클릭 폴백을 생략할지 여부. */
  deferToNativeClosedBy?: boolean;
  /** 열릴 때 처음 초점을 둘 요소 참조. */
  initialFocusRef?: RefObject<HTMLElement | null>;
  /** 바깥 클릭 닫기가 확정되었을 때 부를 함수. */
  onOutsideDismiss: () => void;
}

/**
 * 네이티브 `<dialog>` 모달의 열기·닫기·초점 복귀·바깥 클릭 판별을 공통 처리한다.
 *
 * `open`이 거짓이 되거나 부모가 조건부 언마운트해도 정리 단계에서 `close()`를 부르고
 * 열기 직전 초점이 있던 요소로 키보드 초점을 돌려준다.
 */
export function useModalDialog({
  open,
  allowOutsideDismiss,
  deferToNativeClosedBy = false,
  initialFocusRef,
  onOutsideDismiss,
}: UseModalDialogOptions) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const pressedOutsideRef = useRef(false);
  const closingProgrammaticallyRef = useRef(false);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!open || !dialog) {
      return;
    }

    const previousFocus =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;

    if (!dialog.open) {
      dialog.showModal();
    }
    initialFocusRef?.current?.focus();

    return () => {
      if (dialog.open) {
        closingProgrammaticallyRef.current = true;
        try {
          dialog.close();
        } finally {
          closingProgrammaticallyRef.current = false;
        }
      }
      if (previousFocus?.isConnected) {
        const active = document.activeElement;
        if (
          active === null ||
          active === document.body ||
          dialog.contains(active)
        ) {
          previousFocus.focus();
        }
      }
    };
  }, [open, initialFocusRef]);

  function handlePointerDown(event: PointerEvent<HTMLDialogElement>) {
    const dialog = dialogRef.current;
    if (!dialog || !allowOutsideDismiss || event.target !== dialog) {
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
      !allowOutsideDismiss ||
      !wasOutside ||
      event.target !== dialog
    ) {
      return;
    }
    if (deferToNativeClosedBy && "closedBy" in HTMLDialogElement.prototype) {
      return;
    }
    if (isOutsideRect(dialog, event.clientX, event.clientY)) {
      onOutsideDismiss();
    }
  }

  function isClosingProgrammatically(): boolean {
    return closingProgrammaticallyRef.current;
  }

  return {
    dialogRef,
    isClosingProgrammatically,
    handlePointerDown,
    handlePointerUp,
  };
}
