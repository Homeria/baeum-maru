import type { ReactNode } from "react";
import { ConfirmDialog } from "./ConfirmDialog";

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

/** 지우기 전에 한 번 묻는다. {@link ConfirmDialog}의 지우기용. */
export function DeleteDialog({
  title,
  deleting,
  children,
  onConfirm,
  onCancel,
}: DeleteDialogProps) {
  return (
    <ConfirmDialog
      title={title}
      warning="지우면 되돌릴 수 없습니다."
      confirmLabel="지우기"
      busyLabel="지우는 중…"
      busy={deleting}
      onConfirm={onConfirm}
      onCancel={onCancel}
    >
      {children}
    </ConfirmDialog>
  );
}
