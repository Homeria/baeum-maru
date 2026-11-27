import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { chooseRunMode } from "./api";
import styles from "../../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import { TakeOverPage } from "../handover/TakeOverPage";
import type { ChoosableRunMode } from "./ChangeRunModeDialog";
import runModeStyles from "./runMode.module.css";

/** 처음 켤 때 고르는 것. 넘겨받기는 실행 모드가 아니라 호스트로 시작하는 다른 길이다(BR-H3). */
type Choice = ChoosableRunMode | "take_over";

/** 고를 수 있는 것과 설명. */
const OPTIONS: readonly { mode: Choice; title: string; body: string }[] = [
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
  {
    mode: "take_over",
    title: "넘겨받은 파일로 호스트 시작",
    body: "옛 호스트 PC에서 [호스트 넘기기]로 저장한 파일이 있을 때 고르세요. 이 PC가 새 호스트가 됩니다.",
  },
];

/**
 * 처음 켤 때 실행 모드를 고른다(ADR 0021).
 *
 * 고르면 앱이 기억하고 다시 켜진다. 다음부터는 묻지 않는다.
 */
export function ChooseRunModePage() {
  const [mode, setMode] = useState<Choice | null>(null);
  const [missing, setMissing] = useState(false);
  const [takingOver, setTakingOver] = useState(false);
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
    if (mode === "take_over") {
      setTakingOver(true);
      return;
    }
    submit.mutate(mode);
  };

  if (takingOver) {
    return <TakeOverPage onBack={() => setTakingOver(false)} />;
  }

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
              하나를 골라 주세요
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
