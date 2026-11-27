import { useEffect, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { logout, unlock } from "./api";
import type { Session } from "../../../api/session";
import styles from "../../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import authStyles from "./auth.module.css";
import { loginFormErrorsFrom, type LoginFormErrors } from "./loginFormData";
import { ROLE_LABELS } from "./roles";

/** {@link LockScreen}의 속성. */
export interface LockScreenProps {
  session: Session;
}

/**
 * 화면 잠금 (BR-K8). 메인 화면 위를 덮어 가린다. 뒤의 입력 내용은 그대로 남는다.
 *
 * 작은 창(`<dialog>`)으로 띄운다. 이미 떠 있는 다른 작은 창(지우기 확인 등)보다
 * 나중에 열려 그 위에 오고, 불투명한 바탕이 뒤를 모두 가린다.
 *
 * 같은 직원은 PIN으로 열고, 다른 직원은 로그아웃한 뒤 로그인한다.
 */
export function LockScreen({ session }: LockScreenProps) {
  const queryClient = useQueryClient();
  const [pin, setPin] = useState("");
  const [errors, setErrors] = useState<LoginFormErrors>(NO_FORM_ERRORS);
  const { name, role } = session.staff;
  const dialog = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    dialog.current?.showModal();
  }, []);

  const open = useMutation({
    mutationFn: unlock,
    onSuccess: () => {
      // 잠긴 동안 실패한 목록들을 다시 받는다
      void queryClient.invalidateQueries();
    },
    onError: (error) => {
      setErrors(loginFormErrorsFrom(error));
      setPin("");
    },
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!open.isPending) {
      open.mutate(pin);
    }
  };

  const pinError = errors.fields.pin;

  return (
    <dialog
      ref={dialog}
      className={authStyles.lockOverlay}
      aria-labelledby="lock-title"
      // Esc로 닫히지 않게
      onCancel={(event) => event.preventDefault()}
    >
      <form className={styles.form} onSubmit={submit} noValidate>
        <h2 id="lock-title" className={styles.title}>
          화면이 잠겼습니다
        </h2>
        <p className={authStyles.intro}>
          10분 동안 쓰지 않아 잠갔습니다. <strong>{name}</strong> (
          {ROLE_LABELS[role]})의 PIN을 넣으면 하던 화면으로 돌아갑니다.
        </p>

        <div className={styles.field}>
          <label htmlFor="lock-pin" className={styles.label}>
            PIN
          </label>
          <input
            id="lock-pin"
            className={styles.input}
            type="password"
            inputMode="numeric"
            autoComplete="current-password"
            maxLength={8}
            autoFocus
            value={pin}
            onChange={(event) => {
              setPin(event.target.value);
              setErrors(NO_FORM_ERRORS);
            }}
            aria-invalid={pinError !== undefined}
            aria-describedby={pinError ? "lock-pin-error" : undefined}
          />
          {pinError && (
            <p id="lock-pin-error" className={styles.fieldError}>
              {pinError}
            </p>
          )}
        </div>

        {errors.form && (
          <p className={styles.formError} role="alert">
            {errors.form}
          </p>
        )}

        <div className={styles.actions}>
          <button
            type="submit"
            className={styles.submit}
            disabled={open.isPending}
          >
            {open.isPending ? "확인하는 중…" : "열기"}
          </button>
          <button
            type="button"
            className={styles.secondary}
            disabled={open.isPending}
            onClick={() => void logout()}
          >
            다른 직원으로 로그인
          </button>
        </div>
        <p className={authStyles.hint}>
          다른 직원으로 로그인하면 이 화면에서 쓰던 내용은 사라집니다.
        </p>
      </form>
    </dialog>
  );
}
