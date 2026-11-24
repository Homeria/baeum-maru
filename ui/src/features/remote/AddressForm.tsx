import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { saveHostAddress } from "../../api/remote";
import formStyles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";

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
    <form className={authStyles.stack} onSubmit={onSubmit} noValidate>
      <p className={authStyles.intro}>
        호스트 PC의 "직원 관리 &gt; 기기" 탭에 나온 접속 주소를 넣어 주세요.
      </p>
      <div className={formStyles.field}>
        <label htmlFor="host-address" className={formStyles.label}>
          호스트 주소
        </label>
        <input
          id="host-address"
          className={formStyles.input}
          placeholder="예: 192.168.0.12"
          inputMode="decimal"
          autoComplete="off"
          value={address}
          onChange={(event) => setAddress(event.target.value)}
          aria-invalid={save.isError}
          aria-describedby={save.isError ? "host-address-error" : undefined}
        />
        {save.isError && (
          <p id="host-address-error" className={formStyles.fieldError}>
            {save.error.message}
          </p>
        )}
      </div>
      <div className={formStyles.actions}>
        {onCancel && (
          <button
            type="button"
            className={formStyles.secondary}
            onClick={onCancel}
          >
            그만두기
          </button>
        )}
        <button
          type="submit"
          className={formStyles.submit}
          disabled={save.isPending}
        >
          연결
        </button>
      </div>
    </form>
  );
}
