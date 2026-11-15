import {
  useRef,
  useState,
  type ChangeEvent,
  type FormEvent,
  type ReactNode,
} from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { HostError } from "../../api/client";
import { setup } from "../../api/setup";
import styles from "../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../components/form/formErrors";
import authStyles from "./auth.module.css";
import {
  EMPTY_SETUP_FORM,
  checkPinConfirm,
  firstErrorField,
  setupFormErrorsFrom,
  toSetupRequest,
  type SetupField,
  type SetupFormErrors,
  type SetupFormValues,
} from "./setupFormData";

/** {@link SetupPage}의 속성. */
export interface SetupPageProps {
  /** 관리자를 만들었다. `recoveryCode`는 한 번만 받는 복구 코드. */
  onDone: (recoveryCode: string) => void;
}

/** 처음 설정: 첫 관리자 계정을 만든다 (BR-K11). 직원이 하나도 없을 때만 뜬다. */
export function SetupPage({ onDone }: SetupPageProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<SetupFormValues>(EMPTY_SETUP_FORM);
  const [errors, setErrors] = useState<SetupFormErrors>(NO_FORM_ERRORS);
  const inputs = useRef<Partial<Record<SetupField, HTMLInputElement | null>>>(
    {},
  );

  const showErrors = (next: SetupFormErrors) => {
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (values: SetupFormValues) => setup(toSetupRequest(values)),
    onSuccess: ({ recoveryCode }) => {
      void queryClient.invalidateQueries({ queryKey: ["setupStatus"] });
      onDone(recoveryCode);
    },
    onError: (error) => {
      showErrors(setupFormErrorsFrom(error));
      // 다른 PC에서 먼저 설정했으면 다음 화면으로 넘어가게 다시 묻는다
      if (error instanceof HostError && error.code === "SETUP_ALREADY_DONE") {
        void queryClient.invalidateQueries({ queryKey: ["setupStatus"] });
      }
    },
  });

  const change = (field: SetupField, value: string) => {
    setValues((current) => ({ ...current, [field]: value }));
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (save.isPending) {
      return;
    }
    const mismatch = checkPinConfirm(values);
    if (mismatch) {
      showErrors(mismatch);
      return;
    }
    save.mutate(values);
  };

  /** 칸 하나: 이름표, 입력 부품, 도움말, 오류. */
  const field = (
    name: SetupField,
    label: string,
    hint: string | null,
    control: (props: {
      id: string;
      className: string;
      value: string;
      onChange: (event: ChangeEvent<HTMLInputElement>) => void;
      "aria-invalid": boolean;
      "aria-describedby": string | undefined;
    }) => ReactNode,
  ) => {
    const id = `setup-${name}`;
    const hintId = `${id}-hint`;
    const errorId = `${id}-error`;
    const error = errors.fields[name];
    const describedBy = [hint ? hintId : null, error ? errorId : null]
      .filter((value) => value !== null)
      .join(" ");
    return (
      <div className={styles.field}>
        <label htmlFor={id} className={styles.label}>
          {label}
          <span className={styles.required}>필수</span>
        </label>
        {control({
          id,
          className: styles.input,
          value: values[name],
          onChange: (event) => change(name, event.target.value),
          "aria-invalid": error !== undefined,
          "aria-describedby": describedBy || undefined,
        })}
        {hint && (
          <p id={hintId} className={authStyles.hint}>
            {hint}
          </p>
        )}
        {error && (
          <p id={errorId} className={styles.fieldError}>
            {error}
          </p>
        )}
      </div>
    );
  };

  return (
    <div className={authStyles.page}>
      <form
        className={`${styles.form} ${authStyles.card}`}
        onSubmit={submit}
        noValidate
      >
        <h2 className={styles.title}>배움마루 처음 설정</h2>
        <p className={authStyles.intro}>
          처음 쓰는 것이라 관리자 계정을 먼저 만듭니다. 관리자는 직원 계정을
          만들고 관리합니다.
        </p>

        {field(
          "name",
          "관리자 이름",
          "로그인할 때 이 이름을 고릅니다.",
          (props) => (
            <input
              {...props}
              ref={(element) => {
                inputs.current.name = element;
              }}
              type="text"
              autoComplete="off"
              placeholder="예: 홍길동"
              autoFocus
            />
          ),
        )}
        {field(
          "pin",
          "PIN",
          "숫자 6~8자리. 관리자 PIN은 6자리 이상입니다.",
          (props) => (
            <input
              {...props}
              ref={(element) => {
                inputs.current.pin = element;
              }}
              // 숫자 자판으로, 화면에는 가려서
              type="password"
              inputMode="numeric"
              autoComplete="new-password"
              maxLength={8}
            />
          ),
        )}
        {field("pinConfirm", "PIN 한 번 더", null, (props) => (
          <input
            {...props}
            ref={(element) => {
              inputs.current.pinConfirm = element;
            }}
            type="password"
            inputMode="numeric"
            autoComplete="new-password"
            maxLength={8}
          />
        ))}

        {errors.form && (
          <p className={styles.formError} role="alert">
            {errors.form}
          </p>
        )}

        <div className={styles.actions}>
          <button
            type="submit"
            className={styles.submit}
            disabled={save.isPending}
          >
            {save.isPending ? "만드는 중…" : "관리자 만들기"}
          </button>
        </div>
      </form>
    </div>
  );
}
