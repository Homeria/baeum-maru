import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { cancelHandover, handedOverAdmins } from "../../api/handover";
import { ConfirmDialog } from "../../components/dialog/ConfirmDialog";
import formStyles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import { AdminPinFields } from "./AdminPinFields";

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
  const admins = useQuery({
    queryKey: ["handedOverAdmins"],
    queryFn: handedOverAdmins,
  });
  const cancel = useMutation({
    mutationFn: (id: number) => cancelHandover(id, pin),
    // 틀린 PIN은 지우고 다시 넣게
    onError: () => setPin(""),
  });

  return (
    <ConfirmDialog
      title="넘기기를 취소하고 이 PC를 다시 호스트로 쓸까요?"
      warning="다른 PC에서 이미 넘기기 파일로 호스트를 시작했다면 취소하지 마세요. 호스트가 둘이 되어 접수 명단이 갈라집니다. 넘기기 파일을 잃어버렸거나 아직 아무 PC에서도 시작하지 않았을 때만 취소해 주세요."
      confirmLabel="취소하고 다시 켜기"
      busyLabel="되돌리는 중…"
      busy={cancel.isPending}
      onConfirm={() => {
        if (staffId === null || pin === "") {
          setMissing(true);
          return;
        }
        cancel.mutate(staffId);
      }}
      onCancel={onCancel}
    >
      {admins.isPending ? (
        <p className={authStyles.hint}>관리자 목록을 불러오는 중…</p>
      ) : admins.isError ? (
        <p className={formStyles.formError} role="alert">
          {admins.error.message}
        </p>
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
        <p className={formStyles.formError} role="alert">
          {cancel.error.message}
        </p>
      )}
    </ConfirmDialog>
  );
}
