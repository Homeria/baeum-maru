import { useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { StaffResponse } from "../../api/generated/StaffResponse";
import { resetStaffPin } from "../../api/staff";
import styles from "../../components/form/form.module.css";
import {
  NO_FORM_ERRORS,
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";
import authStyles from "../auth/auth.module.css";
import { pinConfirmError } from "../auth/pinConfirm";

type PinField = "pin" | "pinConfirm";

/** {@link PinResetForm}의 속성. */
export interface PinResetFormProps {
  staff: StaffResponse;
}

/**
 * PIN 초기화 (BR-K10): 직원이 PIN을 잊었을 때 관리자가 새로 정해 준다.
 *
 * 계정이 잠겼어도 새로 정하면 풀린다. 관리자 자신의 PIN도 여기서 바꾼다.
 */
export function PinResetForm({ staff }: PinResetFormProps) {
  const queryClient = useQueryClient();
  const [pin, setPin] = useState("");
  const [pinConfirm, setPinConfirm] = useState("");
  const [errors, setErrors] = useState<FormErrors<PinField>>(NO_FORM_ERRORS);
  const [notice, setNotice] = useState<string | null>(null);

  const reset = useMutation({
    mutationFn: (pin: string) => resetStaffPin(staff.id, pin),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
      setPin("");
      setPinConfirm("");
      setErrors(NO_FORM_ERRORS);
      setNotice(
        `${staff.name}의 PIN을 새로 정했습니다. 잠겨 있었다면 풀렸습니다.`,
      );
    },
    onError: (error) => {
      setErrors(formErrorsFrom(error, ["pin", "pinConfirm"]));
    },
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (reset.isPending) {
      return;
    }
    setNotice(null);
    const mismatch = pinConfirmError(pin, pinConfirm);
    if (mismatch) {
      setErrors({ fields: { pinConfirm: mismatch }, form: null });
      return;
    }
    reset.mutate(pin);
  };

  const pinError = errors.fields.pin;
  const confirmError = errors.fields.pinConfirm;
  const minimum = staff.role === "admin" ? "6~8자리" : "4~8자리";

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>PIN 새로 정하기</h2>
      <p className={authStyles.hint}>
        {staff.name}이(가) PIN을 잊었거나 계정이 잠겼을 때 씁니다. 숫자{" "}
        {minimum}.
      </p>

      <div className={styles.field}>
        <label htmlFor="reset-pin" className={styles.label}>
          새 PIN
        </label>
        <input
          id="reset-pin"
          className={styles.input}
          type="password"
          inputMode="numeric"
          autoComplete="new-password"
          maxLength={8}
          value={pin}
          onChange={(event) => {
            setPin(event.target.value);
            setErrors(NO_FORM_ERRORS);
            setNotice(null);
          }}
          aria-invalid={pinError !== undefined}
          aria-describedby={pinError ? "reset-pin-error" : undefined}
        />
        {pinError && (
          <p id="reset-pin-error" className={styles.fieldError}>
            {pinError}
          </p>
        )}
      </div>

      <div className={styles.field}>
        <label htmlFor="reset-pin-confirm" className={styles.label}>
          새 PIN 한 번 더
        </label>
        <input
          id="reset-pin-confirm"
          className={styles.input}
          type="password"
          inputMode="numeric"
          autoComplete="new-password"
          maxLength={8}
          value={pinConfirm}
          onChange={(event) => {
            setPinConfirm(event.target.value);
            setErrors(NO_FORM_ERRORS);
            setNotice(null);
          }}
          aria-invalid={confirmError !== undefined}
          aria-describedby={
            confirmError ? "reset-pin-confirm-error" : undefined
          }
        />
        {confirmError && (
          <p id="reset-pin-confirm-error" className={styles.fieldError}>
            {confirmError}
          </p>
        )}
      </div>

      {errors.form && (
        <p className={styles.formError} role="alert">
          {errors.form}
        </p>
      )}
      {notice && (
        <p className={styles.notice} role="status">
          {notice}
        </p>
      )}

      <div className={styles.actions}>
        <button
          type="submit"
          className={styles.secondary}
          disabled={reset.isPending}
        >
          {reset.isPending ? "정하는 중…" : "PIN 새로 정하기"}
        </button>
      </div>
    </form>
  );
}
