import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { restartApp } from "../../../api/appCommands";
import { startHandover } from "./api";
import { ConfirmDialog } from "../../../components/dialog/ConfirmDialog";
import formStyles from "../../../components/form/form.module.css";

/** {@link HandoverDialog}의 속성. */
export interface HandoverDialogProps {
  onCancel: () => void;
}

/**
 * 호스트 넘기기를 시작하기 전에 묻는다(BR-H1). 관리자 PIN을 한 번 더 받는다.
 *
 * 시작하면 앱이 다시 켜지고, 서버를 끈 채 넘기기 파일을 만든다.
 */
export function HandoverDialog({ onCancel }: HandoverDialogProps) {
  const [pin, setPin] = useState("");
  const [missing, setMissing] = useState(false);
  const start = useMutation({
    mutationFn: async () => {
      await startHandover(pin);
      await restartApp();
    },
    // 틀린 PIN은 지우고 다시 넣게
    onError: () => setPin(""),
  });

  return (
    <ConfirmDialog
      title="이 PC의 호스트를 다른 PC로 넘길까요?"
      warning="넘기는 동안 다른 PC에서는 배움마루를 쓸 수 없습니다. 다른 PC의 직원에게 저장을 마쳐 달라고 먼저 알려 주세요. 넘기기 파일을 저장하면 이 PC는 접수를 받지 않고, 새 호스트 PC에서 그 파일로 시작합니다."
      confirmLabel="넘기기 시작"
      busyLabel="시작하는 중…"
      busy={start.isPending}
      onConfirm={() => {
        if (pin === "") {
          setMissing(true);
          return;
        }
        start.mutate();
      }}
      onCancel={onCancel}
    >
      <div className={formStyles.field}>
        <label htmlFor="handover-pin" className={formStyles.label}>
          내 PIN
        </label>
        <input
          id="handover-pin"
          className={formStyles.input}
          type="password"
          inputMode="numeric"
          autoComplete="off"
          maxLength={8}
          value={pin}
          onChange={(event) => {
            setPin(event.target.value);
            setMissing(false);
          }}
        />
        {missing && <p className={formStyles.fieldError}>PIN을 넣어 주세요</p>}
      </div>
      {start.isError && (
        <p className={formStyles.formError} role="alert">
          {start.error.message}
        </p>
      )}
    </ConfirmDialog>
  );
}
