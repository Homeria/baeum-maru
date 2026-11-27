import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import authStyles from "../auth/auth.module.css";
import { saveHostAddress } from "./api";

/** {@link AddressForm}의 속성. */
export interface AddressFormProps {
  /** 지금 기억한 주소. 처음이면 `null`. */
  current: string | null;
  onSaved: () => Promise<void>;
  /** 처음 넣을 때는 그만둘 수 없어서 `null`. */
  onCancel: (() => void) | null;
}

/** 호스트 주소를 넣는다(ADR 0004). 앱이 기억해서 다음부터는 묻지 않는다. */
export function AddressForm({ current, onSaved, onCancel }: AddressFormProps) {
  const [address, setAddress] = useState(current ?? "");
  const save = useMutation({
    mutationFn: saveHostAddress,
    onSuccess: onSaved,
  });

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!save.isPending) {
      save.mutate(address);
    }
  };

  return (
    <form className={authStyles.form} onSubmit={onSubmit} noValidate>
      <p className={authStyles.intro}>
        호스트 PC의 &quot;직원 관리 &gt; 기기&quot; 탭에 나온 접속 주소를 넣어
        주세요.
      </p>
      <TextField
        id="host-address"
        label="호스트 주소"
        placeholder="예: 192.168.0.12"
        inputMode="decimal"
        autoComplete="off"
        autoFocus
        value={address}
        error={save.isError ? save.error.message : undefined}
        onChange={(event) => setAddress(event.target.value)}
      />
      <div className={authStyles.actions}>
        {onCancel && (
          <Button
            variant="secondary"
            disabled={save.isPending}
            onClick={onCancel}
          >
            그만두기
          </Button>
        )}
        <Button
          type="submit"
          variant="primary"
          busy={save.isPending}
          busyLabel="연결하는 중…"
        >
          연결
        </Button>
      </div>
    </form>
  );
}
