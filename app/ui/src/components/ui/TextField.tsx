import { useId, type InputHTMLAttributes, type Ref } from "react";
import { StatusPill } from "./StatusPill";
import styles from "./field.module.css";

/** {@link TextField}의 속성. */
export interface TextFieldProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "children"
> {
  /** 칸 이름. `<label htmlFor>`로 입력 칸과 직접 연결된다. */
  label: string;
  /** 입력 칸 위에 보여 줄 짧은 도움말. */
  hint?: string;
  /** 검증 실패 문구. 값이 있을 때만 `aria-invalid="true"`를 붙인다. */
  error?: string;
  /** 내부 `<input>` 참조. */
  ref?: Ref<HTMLInputElement>;
}

/** 라벨·필수 꼬리표·도움말·오류 문구를 함께 묶은 한 줄 입력 칸. */
export function TextField({
  label,
  hint,
  error,
  required = false,
  id,
  className,
  ref,
  "aria-describedby": describedByProp,
  ...rest
}: TextFieldProps) {
  const autoId = useId();
  const inputId = id ?? autoId;
  const hintId = hint ? `${inputId}-hint` : undefined;
  const errorId = error ? `${inputId}-error` : undefined;
  const describedBy =
    [describedByProp, hintId, errorId].filter(Boolean).join(" ") || undefined;

  return (
    <div className={[styles.field, className].filter(Boolean).join(" ")}>
      <div className={styles.labelRow}>
        <label htmlFor={inputId} className={styles.label}>
          {label}
        </label>
        {required && <StatusPill tone="bad">필수</StatusPill>}
      </div>
      {hint && (
        <p id={hintId} className={styles.hint}>
          {hint}
        </p>
      )}
      <input
        {...rest}
        ref={ref}
        id={inputId}
        required={required}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy}
        className={styles.control}
      />
      {error && (
        <p id={errorId} className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
