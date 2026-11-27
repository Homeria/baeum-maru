import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { HostError } from "../../../api/client";
import { listLoginStaff, login } from "./api";
import styles from "../../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import authStyles from "./auth.module.css";
import { ROLE_LABELS } from "./roles";
import {
  groupByRole,
  loginFormErrorsFrom,
  toLoginRequest,
  type LoginFormErrors,
} from "./loginFormData";

/** {@link LoginPage}의 속성. */
export interface LoginPageProps {
  /** "PIN을 잊었어요"를 눌렀다. */
  onForgotPin: () => void;
}

/**
 * 로그인: 이름을 고르고 PIN을 넣는다 (BR-K1, BR-K2).
 *
 * 이름은 역할별로 묶어 보여 준다. 임시 직원이 많아도 찾기 쉽게.
 *
 * 성공하면 세션이 생겨 앱이 메인 화면으로 바뀐다.
 */
export function LoginPage({ onForgotPin }: LoginPageProps) {
  const queryClient = useQueryClient();
  const staffList = useQuery({
    queryKey: ["loginStaff"],
    queryFn: listLoginStaff,
  });
  const [staffId, setStaffId] = useState<number | null>(null);
  const [pin, setPin] = useState("");
  const [errors, setErrors] = useState<LoginFormErrors>(NO_FORM_ERRORS);
  const pinInput = useRef<HTMLInputElement>(null);

  const submit = useMutation({
    mutationFn: login,
    onError: (error) => {
      setErrors(loginFormErrorsFrom(error));
      // 틀린 PIN은 지우고 다시 넣게
      setPin("");
      pinInput.current?.focus();
      // 그사이 사용 중지되었거나 없어진 직원은 목록에서 빠지게
      if (
        error instanceof HostError &&
        (error.code === "STAFF_NOT_FOUND" ||
          error.code === "ACCOUNT_UNAVAILABLE")
      ) {
        void queryClient.invalidateQueries({ queryKey: ["loginStaff"] });
      }
    },
  });

  const choose = (id: number) => {
    setStaffId(id);
    setErrors(NO_FORM_ERRORS);
    pinInput.current?.focus();
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (submit.isPending) {
      return;
    }
    const request = toLoginRequest(staffId, pin);
    if ("fields" in request) {
      setErrors(request);
      return;
    }
    submit.mutate(request);
  };

  if (staffList.isPending) {
    return <p className={styles.loading}>직원 목록을 불러오는 중…</p>;
  }
  if (staffList.isError) {
    return (
      <p className={styles.loading} role="alert">
        {staffList.error.message}
      </p>
    );
  }

  const items = staffList.data.items;
  const staffError = errors.fields.staffId;
  const pinError = errors.fields.pin;

  return (
    <div className={authStyles.page}>
      <form
        className={`${styles.form} ${authStyles.card}`}
        onSubmit={onSubmit}
        noValidate
      >
        <h2 className={styles.title}>배움마루 로그인</h2>

        {items.length === 0 ? (
          <p className={styles.formError} role="alert">
            로그인할 수 있는 직원이 없습니다. 모든 계정이 사용 중지되었거나
            유효기간이 지났습니다.
          </p>
        ) : (
          <fieldset
            className={styles.field}
            aria-invalid={staffError !== undefined}
            aria-describedby={staffError ? "login-staff-error" : undefined}
          >
            <legend className={styles.label}>이름</legend>
            {groupByRole(items).map((group) => (
              <div
                key={group.role}
                role="group"
                aria-labelledby={`login-role-${group.role}`}
                className={authStyles.roleGroup}
              >
                <p
                  id={`login-role-${group.role}`}
                  className={authStyles.roleTitle}
                >
                  {ROLE_LABELS[group.role]}
                </p>
                <div className={authStyles.names}>
                  {group.items.map((item) => (
                    <label key={item.id} className={authStyles.name}>
                      <input
                        type="radio"
                        name="staff"
                        value={item.id}
                        checked={staffId === item.id}
                        onChange={() => choose(item.id)}
                      />
                      {item.name}
                    </label>
                  ))}
                </div>
              </div>
            ))}
            {staffError && (
              <p id="login-staff-error" className={styles.fieldError}>
                {staffError}
              </p>
            )}
          </fieldset>
        )}

        <div className={styles.field}>
          <label htmlFor="login-pin" className={styles.label}>
            PIN
          </label>
          <input
            ref={pinInput}
            id="login-pin"
            className={styles.input}
            type="password"
            inputMode="numeric"
            autoComplete="current-password"
            maxLength={8}
            value={pin}
            onChange={(event) => {
              setPin(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, pin: undefined },
              }));
            }}
            aria-invalid={pinError !== undefined}
            aria-describedby={pinError ? "login-pin-error" : undefined}
          />
          {pinError && (
            <p id="login-pin-error" className={styles.fieldError}>
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
            disabled={submit.isPending || items.length === 0}
          >
            {submit.isPending ? "확인하는 중…" : "로그인"}
          </button>
          <button
            type="button"
            className={styles.secondary}
            onClick={onForgotPin}
            disabled={submit.isPending}
          >
            PIN을 잊었어요
          </button>
        </div>
      </form>
    </div>
  );
}
