import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { listLoginStaff } from "../../api/auth";
import { recover } from "../../api/recovery";
import styles from "../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../components/form/formErrors";
import authStyles from "./auth.module.css";
import {
  EMPTY_RECOVERY_FORM,
  firstErrorField,
  recoveryFormErrorsFrom,
  toRecoveryRequest,
  type RecoveryField,
  type RecoveryFormErrors,
  type RecoveryFormValues,
} from "./recoveryFormData";

/** {@link RecoveryPage}의 속성. */
export interface RecoveryPageProps {
  /** 로그인 화면으로 돌아간다. */
  onBack: () => void;
  /** PIN을 다시 정했다. `recoveryCode`는 새 복구 코드. */
  onDone: (recoveryCode: string) => void;
}

/**
 * PIN을 잊었을 때 (BR-K10, BR-K12).
 *
 * 직원은 관리자에게 PIN 새로 정하기를 부탁하고, 관리자는 인쇄해 둔 복구 코드로 PIN을 다시 정한다.
 * 다시 정하면 새 복구 코드가 나온다.
 */
export function RecoveryPage({ onBack, onDone }: RecoveryPageProps) {
  const staffList = useQuery({
    queryKey: ["loginStaff"],
    queryFn: listLoginStaff,
  });
  const [values, setValues] = useState<RecoveryFormValues>(EMPTY_RECOVERY_FORM);
  const [errors, setErrors] = useState<RecoveryFormErrors>(NO_FORM_ERRORS);
  const inputs = useRef<
    Partial<Record<RecoveryField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const showErrors = (next: RecoveryFormErrors) => {
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field !== null) {
      inputs.current[field]?.focus();
    }
  };

  const submit = useMutation({
    mutationFn: recover,
    onSuccess: (response) => onDone(response.recoveryCode),
    onError: (error) => showErrors(recoveryFormErrorsFrom(error)),
  });

  const change = <F extends keyof RecoveryFormValues>(
    field: F,
    value: RecoveryFormValues[F],
  ) => {
    setValues((current) => ({ ...current, [field]: value }));
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (submit.isPending) {
      return;
    }
    const request = toRecoveryRequest(values);
    if ("fields" in request) {
      showErrors(request);
      return;
    }
    submit.mutate(request);
  };

  const textField = (
    field: "code" | "pin" | "pinConfirm",
    label: string,
    secret: boolean,
  ) => {
    const errorId = `recovery-${field}-error`;
    const error = errors.fields[field];
    return (
      <div className={styles.field}>
        <label htmlFor={`recovery-${field}`} className={styles.label}>
          {label}
        </label>
        <input
          id={`recovery-${field}`}
          ref={(element) => {
            inputs.current[field] = element;
          }}
          className={styles.input}
          type={secret ? "password" : "text"}
          inputMode={secret ? "numeric" : undefined}
          autoComplete={secret ? "new-password" : "off"}
          maxLength={secret ? 8 : undefined}
          value={values[field]}
          onChange={(event) => change(field, event.target.value)}
          aria-invalid={error !== undefined}
          aria-describedby={error ? errorId : undefined}
        />
        {error && (
          <p id={errorId} className={styles.fieldError}>
            {error}
          </p>
        )}
      </div>
    );
  };

  const staffError = errors.fields.staffId;

  return (
    <div className={authStyles.page}>
      <form
        className={`${styles.form} ${authStyles.card}`}
        onSubmit={onSubmit}
        noValidate
      >
        <h2 className={styles.title}>PIN을 잊었을 때</h2>
        <p className={authStyles.intro}>
          직원과 임시 직원은 관리자에게 PIN을 새로 정해 달라고 부탁해 주세요.
        </p>
        <p className={authStyles.intro}>
          관리자는 처음 설정 때 인쇄해 둔 복구 코드로 PIN을 다시 정합니다. 다시
          정하면 새 복구 코드가 나오고, 쓴 코드는 더 이상 쓸 수 없습니다.
        </p>

        {staffList.isError ? (
          <p className={styles.formError} role="alert">
            {staffList.error.message}
          </p>
        ) : (
          <div className={styles.field}>
            <label htmlFor="recovery-staffId" className={styles.label}>
              관리자 이름
            </label>
            <select
              id="recovery-staffId"
              ref={(element) => {
                inputs.current.staffId = element;
              }}
              className={styles.input}
              value={values.staffId ?? ""}
              onChange={(event) =>
                change(
                  "staffId",
                  event.target.value === "" ? null : Number(event.target.value),
                )
              }
              disabled={staffList.isPending}
              aria-invalid={staffError !== undefined}
              aria-describedby={
                staffError ? "recovery-staffId-error" : undefined
              }
            >
              <option value="">
                {staffList.isPending ? "불러오는 중…" : "골라 주세요"}
              </option>
              {staffList.data?.items.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.name}
                </option>
              ))}
            </select>
            {staffError && (
              <p id="recovery-staffId-error" className={styles.fieldError}>
                {staffError}
              </p>
            )}
          </div>
        )}

        {textField("code", "복구 코드", false)}
        <p className={authStyles.hint}>
          예: K7QF-M2XA-9TRE-HW4C. 가운데 줄표는 없어도 됩니다.
        </p>
        {textField("pin", "새 PIN (숫자 6~8자리)", true)}
        {textField("pinConfirm", "새 PIN 한 번 더", true)}

        {errors.form && (
          <p className={styles.formError} role="alert">
            {errors.form}
          </p>
        )}

        <div className={styles.actions}>
          <button
            type="submit"
            className={styles.submit}
            disabled={submit.isPending}
          >
            {submit.isPending ? "확인하는 중…" : "PIN 다시 정하기"}
          </button>
          <button
            type="button"
            className={styles.secondary}
            onClick={onBack}
            disabled={submit.isPending}
          >
            로그인으로 돌아가기
          </button>
        </div>
      </form>
    </div>
  );
}
