import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { listLoginStaff } from "../auth/api";
import type { RunMode } from "../../../api/generated/RunMode";
import { changeRunMode, permitRunModeChange } from "./api";
import { ConfirmDialog } from "../../../components/dialog/ConfirmDialog";
import formStyles from "../../../components/form/form.module.css";
import { AdminPinFields } from "../auth/AdminPinFields";
import authStyles from "../auth/auth.module.css";

/** 직원이 고를 수 있는 실행 모드. 넘기기 상태는 넘기기 기능으로만 들어간다(BR-H1). */
export type ChoosableRunMode = Extract<RunMode, "host" | "remote">;

/** 바꿀 모드마다 묻는 말과 경고. */
const TEXTS: Record<ChoosableRunMode, { title: string; warning: string }> = {
  host: {
    title: "이 PC를 호스트로 바꿀까요?",
    warning:
      "이 PC에 데이터를 두고 다른 PC들이 이 PC에 접속하게 됩니다. 지금 쓰던 호스트의 데이터는 이 PC로 옮겨지지 않습니다. 호스트가 둘이 되면 접수 명단이 갈라지니, 호스트 PC를 바꾸려는 게 아니면 그만두세요.",
  },
  remote: {
    title: "이 PC를 다른 PC에 접속하는 모드로 바꿀까요?",
    warning:
      "이 PC의 데이터는 지우지 않지만, 접속 모드에서는 쓰지 않습니다. 다시 호스트로 바꾸면 그 데이터로 돌아옵니다.",
  },
};

/** {@link ChangeRunModeDialog}의 속성. */
export interface ChangeRunModeDialogProps {
  /** 바꿀 모드. */
  target: ChoosableRunMode;
  /** 관리자 PIN을 확인해야 하는지(BR-K17). 지킬 데이터가 아직 없으면 `false`. */
  needsPin: boolean;
  onCancel: () => void;
}

/**
 * 실행 모드를 바꾸기 전에 묻는다(ADR 0021). 바꾸면 앱이 다시 켜진다.
 *
 * `needsPin`이면 관리자를 고르고 PIN을 넣어 호스트의 허락을 먼저 받는다.
 */
export function ChangeRunModeDialog({
  target,
  needsPin,
  onCancel,
}: ChangeRunModeDialogProps) {
  const [staffId, setStaffId] = useState<number | null>(null);
  const [pin, setPin] = useState("");
  const [missing, setMissing] = useState(false);
  const admins = useQuery({
    queryKey: ["loginStaff"],
    queryFn: listLoginStaff,
    enabled: needsPin,
    select: (list) => list.items.filter((item) => item.role === "admin"),
  });
  const change = useMutation({
    mutationFn: async () => {
      if (needsPin && staffId !== null) {
        await permitRunModeChange(staffId, pin);
      }
      await changeRunMode(target);
    },
    // 틀린 PIN은 지우고 다시 넣게
    onError: () => setPin(""),
  });

  const confirm = () => {
    if (needsPin && (staffId === null || pin === "")) {
      setMissing(true);
      return;
    }
    change.mutate();
  };

  const text = TEXTS[target];
  return (
    <ConfirmDialog
      title={text.title}
      warning={text.warning}
      confirmLabel="바꾸고 다시 켜기"
      busyLabel="바꾸는 중…"
      busy={change.isPending}
      onConfirm={confirm}
      onCancel={onCancel}
    >
      {needsPin && (
        <div className={authStyles.stack}>
          <p className={authStyles.hint}>관리자 PIN이 필요합니다.</p>
          {admins.isPending ? (
            <p className={authStyles.hint}>관리자 목록을 불러오는 중…</p>
          ) : admins.isError ? (
            <p className={formStyles.formError} role="alert">
              {admins.error.message}
            </p>
          ) : (
            <AdminPinFields
              idPrefix="run-mode"
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
        </div>
      )}
      {change.isError && (
        <p className={formStyles.formError} role="alert">
          {change.error.message}
        </p>
      )}
    </ConfirmDialog>
  );
}
