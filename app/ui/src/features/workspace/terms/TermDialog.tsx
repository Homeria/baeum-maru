import { useRef } from "react";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { TermForm, type TermFormProps } from "./TermForm";

/** {@link TermDialog}의 속성. 그만두기가 늘 있다. */
export type TermDialogProps = Omit<TermFormProps, "nameInputRef"> & {
  onCancel: () => void;
};

/**
 * 학기 입력 칸을 모달 대화상자로 띄운다.
 *
 * 창이 떠 있는 동안 뒤쪽 화면은 비활성(`inert`) 처리되며, Esc를 누르면 그만둔다.
 */
export function TermDialog(props: TermDialogProps) {
  const nameInputRef = useRef<HTMLInputElement>(null);

  return (
    <ModalDialog
      open
      size="md"
      title={props.editing ? "학기 고치기" : "새 학기 등록"}
      initialFocusRef={nameInputRef}
      onClose={props.onCancel}
    >
      <TermForm {...props} nameInputRef={nameInputRef} />
    </ModalDialog>
  );
}
