import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import authStyles from "../auth/auth.module.css";
import { saveDeviceRequest } from "./api";
import { requestApproval } from "./approvalApi";

/** 기기 이름의 가장 긴 글자 수. 호스트의 `DeviceName::MAX_CHARS`와 같다. */
const MAX_NAME_CHARS = 40;

/** {@link RequestApproval}의 속성. */
export interface RequestApprovalProps {
  /** 미리 채울 이 PC의 이름. 찾지 못했으면 `null`. */
  computerName: string | null;
  onRequested: () => Promise<void>;
}

/** 이 PC의 승인을 호스트에 요청한다(BR-K15). 받은 기기 증표와 번호는 앱에 맡긴다. */
export function RequestApproval({
  computerName,
  onRequested,
}: RequestApprovalProps) {
  const [name, setName] = useState(computerName ?? "");
  const submit = useMutation({
    mutationFn: async () => {
      const { token, code } = await requestApproval(name);
      await saveDeviceRequest(token, code);
    },
    onSuccess: onRequested,
  });

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!submit.isPending) {
      submit.mutate();
    }
  };

  return (
    <form className={authStyles.form} onSubmit={onSubmit} noValidate>
      <p className={authStyles.intro}>
        이 PC는 아직 승인받지 않았습니다. 승인을 요청하면 호스트 PC 화면에 이
        PC의 이름과 번호가 뜹니다.
      </p>
      <TextField
        id="device-name"
        label="이 PC의 이름"
        hint="관리자가 어느 PC인지 알아볼 수 있게 적어 주세요."
        placeholder="예: 접수 노트북"
        maxLength={MAX_NAME_CHARS}
        autoComplete="off"
        value={name}
        onChange={(event) => setName(event.target.value)}
      />
      {submit.isError && (
        <NoticeBanner tone="error" message={submit.error.message} />
      )}
      <div className={authStyles.actions}>
        <Button
          type="submit"
          variant="primary"
          busy={submit.isPending}
          busyLabel="요청하는 중…"
        >
          승인 요청
        </Button>
      </div>
    </form>
  );
}
