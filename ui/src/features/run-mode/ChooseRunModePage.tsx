import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import type { RunMode } from "../../api/generated/RunMode";
import { chooseRunMode } from "../../api/runMode";
import styles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import runModeStyles from "./runMode.module.css";

/** 고를 수 있는 실행 모드와 설명. */
const OPTIONS: readonly { mode: RunMode; title: string; body: string }[] = [
  {
    mode: "host",
    title: "이 PC에서 새로 시작",
    body: "이 PC에 데이터를 둡니다. 기관에서 처음 설치하는 PC라면 이것을 고르세요.",
  },
  {
    mode: "remote",
    title: "다른 PC에 접속",
    body: "이미 배움마루를 쓰는 PC(호스트)에 접속해 같은 데이터를 씁니다.",
  },
];

/**
 * 처음 켤 때 실행 모드를 고른다(ADR 0021).
 *
 * 고르면 앱이 기억하고 다시 켜진다. 다음부터는 묻지 않는다.
 */
export function ChooseRunModePage() {
  const [mode, setMode] = useState<RunMode | null>(null);
  const [missing, setMissing] = useState(false);
  const submit = useMutation({ mutationFn: chooseRunMode });

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (submit.isPending) {
      return;
    }
    if (mode === null) {
      setMissing(true);
      return;
    }
    submit.mutate(mode);
  };

  return (
    <div className={authStyles.page}>
      <form
        className={`${styles.form} ${authStyles.card}`}
        onSubmit={onSubmit}
        noValidate
      >
        <h2 className={styles.title}>배움마루 시작하기</h2>
        <fieldset
          className={styles.field}
          aria-invalid={missing}
          aria-describedby={missing ? "run-mode-error" : undefined}
        >
          <legend className={styles.label}>이 PC를 어떻게 쓸까요?</legend>
          <div className={runModeStyles.options}>
            {OPTIONS.map((option) => (
              <label key={option.mode} className={runModeStyles.option}>
                <input
                  type="radio"
                  name="run-mode"
                  value={option.mode}
                  checked={mode === option.mode}
                  onChange={() => {
                    setMode(option.mode);
                    setMissing(false);
                  }}
                />
                <span className={runModeStyles.optionTitle}>
                  {option.title}
                </span>
                <span className={runModeStyles.optionBody}>{option.body}</span>
              </label>
            ))}
          </div>
          {missing && (
            <p id="run-mode-error" className={styles.fieldError}>
              둘 중 하나를 골라 주세요
            </p>
          )}
        </fieldset>
        <p className={authStyles.hint}>
          나중에 바꾸려면 관리자 PIN이 필요합니다.
        </p>
        {submit.isError && (
          <p className={styles.formError} role="alert">
            {submit.error.message}
          </p>
        )}
        <div className={styles.actions}>
          <button
            type="submit"
            className={styles.submit}
            disabled={submit.isPending}
          >
            이렇게 시작
          </button>
        </div>
      </form>
    </div>
  );
}
