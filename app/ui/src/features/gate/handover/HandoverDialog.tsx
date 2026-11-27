import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { restartApp } from "../../../api/appCommands";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import { startHandover } from "./api";
import styles from "./HandoverDialog.module.css";

/** {@link HandoverDialog}의 속성. */
export interface HandoverDialogProps {
  onCancel: () => void;
}

/**
 * 호스트 넘기기를 시작하기 전에 묻는 모달 대화상자(BR-H1). 관리자 PIN을 한 번 더 받는다.
 *
 * 시작하면 앱이 다시 켜지고, 서버를 끈 채 넘기기 파일을 만든다.
 */
export function HandoverDialog({ onCancel }: HandoverDialogProps) {
  const formId = useId();
  const [pin, setPin] = useState("");
  const [missing, setMissing] = useState(false);
  const pinInputRef = useRef<HTMLInputElement>(null);

  const start = useMutation({
    mutationFn: async () => {
      await startHandover(pin);
      await restartApp();
    },
    // 틀린 PIN은 지우고 다시 넣게
    onError: () => setPin(""),
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (start.isPending) {
      return;
    }
    if (pin === "") {
      setMissing(true);
      pinInputRef.current?.focus();
      return;
    }
    start.mutate();
  };

  return (
    <ModalDialog
      open
      size="md"
      title="이 PC의 호스트를 다른 PC로 넘길까요?"
      busy={start.isPending}
      initialFocusRef={pinInputRef}
      onClose={onCancel}
      footer={
        <>
          <Button
            variant="secondary"
            disabled={start.isPending}
            onClick={onCancel}
          >
            그만두기
          </Button>
          <Button
            type="submit"
            form={formId}
            variant="dangerSolid"
            busy={start.isPending}
            busyLabel="시작하는 중…"
          >
            넘기기 시작
          </Button>
        </>
      }
    >
      <form id={formId} className={styles.form} onSubmit={submit} noValidate>
        <NoticeBanner
          tone="error"
          message="넘기는 동안 다른 PC에서는 배움마루를 쓸 수 없습니다. 다른 PC의 직원에게 저장을 마쳐 달라고 먼저 알려 주세요. 넘기기 파일을 저장하면 이 PC는 접수를 받지 않고, 새 호스트 PC에서 그 파일로 시작합니다."
        />

        <TextField
          ref={pinInputRef}
          label="내 PIN"
          required
          type="password"
          inputMode="numeric"
          autoComplete="off"
          maxLength={8}
          value={pin}
          onChange={(event) => {
            setPin(event.target.value);
            setMissing(false);
          }}
          error={missing ? "PIN을 넣어 주세요" : undefined}
        />

        {start.isError && (
          <NoticeBanner tone="error" message={start.error.message} />
        )}
      </form>
    </ModalDialog>
  );
}
