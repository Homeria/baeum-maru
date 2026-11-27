import { useRef, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { AdminPinFields } from "../auth/AdminPinFields";
import authStyles from "../auth/auth.module.css";
import { cancelHandover, handedOverAdmins } from "./api";

/** {@link CancelHandoverDialog}의 속성. */
export interface CancelHandoverDialogProps {
  onCancel: () => void;
}

/**
 * 넘기기를 취소하기 전에 묻는다(BR-H2). 이 PC의 관리자를 고르고 PIN을 넣는다.
 *
 * 취소하면 앱이 다시 켜지며 이 PC가 다시 호스트가 된다.
 */
export function CancelHandoverDialog({ onCancel }: CancelHandoverDialogProps) {
  const [staffId, setStaffId] = useState<number | null>(null);
  const [pin, setPin] = useState("");
  const [missing, setMissing] = useState(false);
  const cancelRef = useRef<HTMLButtonElement>(null);
  const admins = useQuery({
    queryKey: ["handedOverAdmins"],
    queryFn: handedOverAdmins,
  });
  const cancel = useMutation({
    mutationFn: (id: number) => cancelHandover(id, pin),
    // 틀린 PIN은 지우고 다시 넣게
    onError: () => setPin(""),
  });

  const confirm = () => {
    if (staffId === null || pin === "") {
      setMissing(true);
      return;
    }
    cancel.mutate(staffId);
  };

  return (
    <ModalDialog
      open
      title="넘기기를 취소하고 이 PC를 다시 호스트로 쓸까요?"
      busy={cancel.isPending}
      initialFocusRef={cancelRef}
      onClose={onCancel}
      footer={
        <>
          <Button
            ref={cancelRef}
            variant="secondary"
            disabled={cancel.isPending}
            onClick={onCancel}
          >
            그만두기
          </Button>
          <Button
            variant="dangerSolid"
            busy={cancel.isPending}
            busyLabel="되돌리는 중…"
            onClick={confirm}
          >
            취소하고 다시 켜기
          </Button>
        </>
      }
    >
      <div className={authStyles.stack}>
        <p className={authStyles.intro}>
          다른 PC에서 이미 넘기기 파일로 호스트를 시작했다면 취소하지 마세요.
          호스트가 둘이 되어 접수 명단이 갈라집니다. 넘기기 파일을 잃어버렸거나
          아직 아무 PC에서도 시작하지 않았을 때만 취소해 주세요.
        </p>
        {admins.isPending ? (
          <p className={authStyles.hint}>관리자 목록을 불러오는 중…</p>
        ) : admins.isError ? (
          <NoticeBanner tone="error" message={admins.error.message} />
        ) : (
          <AdminPinFields
            idPrefix="cancel-handover"
            admins={admins.data}
            staffId={staffId}
            onStaffId={(id) => {
              setStaffId(id);
              setMissing(false);
            }}
            pin={pin}
            onPin={(value) => {
              setPin(value);
              setMissing(false);
            }}
            missing={missing}
          />
        )}
        {cancel.isError && (
          <NoticeBanner tone="error" message={cancel.error.message} />
        )}
      </div>
    </ModalDialog>
  );
}
