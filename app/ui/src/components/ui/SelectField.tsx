import {
  useId,
  type ReactNode,
  type Ref,
  type SelectHTMLAttributes,
} from "react";
import { StatusPill } from "./StatusPill";
import styles from "./field.module.css";

/** {@link SelectField}의 속성. */
export interface SelectFieldProps extends SelectHTMLAttributes<HTMLSelectElement> {
  /** 칸 이름. `<label htmlFor>`로 선택 칸과 직접 연결된다. */
  label: string;
  /** 선택 칸 위에 보여 줄 짧은 도움말. */
  hint?: string;
  /** 검증 실패 문구. 값이 있을 때만 `aria-invalid="true"`를 붙인다. */
  error?: string;
  /** `<option>` 또는 `<optgroup>` 목록. */
  children: ReactNode;
  /** 내부 `<select>` 참조. */
  ref?: Ref<HTMLSelectElement>;
}

/** 라벨·필수 꼬리표·도움말·오류 문구를 함께 묶은 네이티브 선택 칸. */
export function SelectField({
  label,
  hint,
  error,
  required = false,
  id,
  className,
  children,
  ref,
  "aria-describedby": describedByProp,
  ...rest
}: SelectFieldProps) {
  const autoId = useId();
  const selectId = id ?? autoId;
  const hintId = hint ? `${selectId}-hint` : undefined;
  const errorId = error ? `${selectId}-error` : undefined;
  const describedBy =
    [describedByProp, hintId, errorId].filter(Boolean).join(" ") || undefined;

  return (
    <div className={[styles.field, className].filter(Boolean).join(" ")}>
      <div className={styles.labelRow}>
        <label htmlFor={selectId} className={styles.label}>
          {label}
        </label>
        {required && <StatusPill tone="bad">필수</StatusPill>}
      </div>
      {hint && (
        <p id={hintId} className={styles.hint}>
          {hint}
        </p>
      )}
      <select
        {...rest}
        ref={ref}
        id={selectId}
        required={required}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy}
        className={styles.control}
      >
        {children}
      </select>
      {error && (
        <p id={errorId} className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
