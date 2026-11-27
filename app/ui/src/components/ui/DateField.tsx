import {
  useId,
  useRef,
  type InputHTMLAttributes,
  type Ref,
  type RefObject,
} from "react";
import { StatusPill } from "./StatusPill";
import { formatDateInput } from "./dateInput";
import styles from "./field.module.css";

const ISO_DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

/** {@link DateField}의 속성. */
export interface DateFieldProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "type" | "value" | "onChange" | "children"
> {
  /** 칸 이름. `<label htmlFor>`로 입력 칸과 직접 연결된다. */
  label: string;
  /** 현재 날짜 값(`YYYY-MM-DD` 또는 입력 중인 글자). */
  value: string;
  /** 날짜가 바뀔 때 정리된 글자를 넘긴다. */
  onChange: (value: string) => void;
  /** 입력 칸 위에 보여 줄 짧은 도움말. 기본값은 숫자 8자리 형식 안내. */
  hint?: string;
  /** 검증 실패 문구. 값이 있을 때만 `aria-invalid="true"`를 붙인다. */
  error?: string;
  /** 내부 텍스트 `<input>` 참조. */
  ref?: Ref<HTMLInputElement>;
}

/**
 * 숫자 8자리 자동 하이픈 입력과 선택용 달력 버튼을 함께 제공하는 공통 날짜 입력 칸.
 *
 * 생년월일처럼 먼 과거 연도도 키보드로 빠르게 입력할 수 있고, 필요할 때만 오른쪽 달력
 * 버튼으로 네이티브 달력을 열 수 있다.
 */
export function DateField({
  label,
  value,
  onChange,
  hint = "숫자 8자리 입력 (예: 1955-03-15)",
  error,
  required = false,
  placeholder = "예: 1955-03-15",
  disabled = false,
  id,
  className,
  ref,
  "aria-describedby": describedByProp,
  ...rest
}: DateFieldProps) {
  const autoId = useId();
  const inputId = id ?? autoId;
  const hintId = hint ? `${inputId}-hint` : undefined;
  const errorId = error ? `${inputId}-error` : undefined;
  const describedBy =
    [describedByProp, hintId, errorId].filter(Boolean).join(" ") || undefined;

  const localInputRef = useRef<HTMLInputElement>(null);
  const pickerRef = useRef<HTMLInputElement>(null);
  const supportsPicker =
    typeof HTMLInputElement !== "undefined" &&
    typeof HTMLInputElement.prototype.showPicker === "function";

  function setInputRef(node: HTMLInputElement | null) {
    localInputRef.current = node;
    if (typeof ref === "function") {
      ref(node);
    } else if (ref && "current" in ref) {
      (ref as RefObject<HTMLInputElement | null>).current = node;
    }
  }

  function openCalendar() {
    const picker = pickerRef.current;
    if (!picker || !supportsPicker) {
      return;
    }
    try {
      picker.showPicker();
    } catch {
      // 일부 웹뷰 환경에서 팝업이 막히면 바로 키보드로 입력하도록 초점을 돌려준다
      localInputRef.current?.focus();
    }
  }

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
      <div className={styles.dateWrap}>
        <input
          {...rest}
          ref={setInputRef}
          id={inputId}
          type="text"
          inputMode="numeric"
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="none"
          spellCheck={false}
          maxLength={14}
          required={required}
          disabled={disabled}
          placeholder={placeholder}
          value={value}
          onChange={(event) => onChange(formatDateInput(event.target.value))}
          aria-invalid={error ? true : undefined}
          aria-describedby={describedBy}
          className={[
            styles.control,
            supportsPicker && !disabled ? styles.dateInputWithPicker : "",
          ]
            .filter(Boolean)
            .join(" ")}
        />
        {supportsPicker && !disabled && (
          <>
            <input
              ref={pickerRef}
              type="date"
              tabIndex={-1}
              aria-hidden="true"
              className={styles.datePickerProxy}
              value={ISO_DATE_PATTERN.test(value) ? value : ""}
              onChange={(event) => {
                onChange(event.target.value);
                localInputRef.current?.focus();
              }}
            />
            <button
              type="button"
              className={styles.datePickerButton}
              aria-label={`${label} 달력에서 고르기`}
              title="달력에서 고르기"
              onClick={openCalendar}
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 20 20"
                fill="none"
                aria-hidden="true"
              >
                <rect
                  x="2.75"
                  y="4.25"
                  width="14.5"
                  height="13"
                  rx="2"
                  stroke="currentColor"
                  strokeWidth="1.6"
                />
                <line
                  x1="2.75"
                  y1="8.25"
                  x2="17.25"
                  y2="8.25"
                  stroke="currentColor"
                  strokeWidth="1.6"
                />
                <line
                  x1="6.75"
                  y1="2.5"
                  x2="6.75"
                  y2="5.75"
                  stroke="currentColor"
                  strokeWidth="1.6"
                  strokeLinecap="round"
                />
                <line
                  x1="13.25"
                  y1="2.5"
                  x2="13.25"
                  y2="5.75"
                  stroke="currentColor"
                  strokeWidth="1.6"
                  strokeLinecap="round"
                />
              </svg>
            </button>
          </>
        )}
      </div>
      {error && (
        <p id={errorId} className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
