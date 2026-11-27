import {
  useId,
  useRef,
  type InputHTMLAttributes,
  type Ref,
  type RefObject,
} from "react";
import styles from "./field.module.css";

/** {@link SearchInput}의 속성. */
export interface SearchInputProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "type" | "value" | "onChange" | "children"
> {
  /** 검색 칸 이름. `hideLabel`이 참이어도 화면 읽기 프로그램에는 그대로 읽힌다. */
  label: string;
  /** 현재 검색어. */
  value: string;
  /** 검색어가 바뀔 때 호출된다. */
  onChange: (value: string) => void;
  /** 화면에서 라벨을 숨길지 여부. 도구 막대처럼 문맥이 분명할 때 쓴다. */
  hideLabel?: boolean;
  /** 지우기 버튼의 접근성 이름. 기본값은 "검색어 지우기". */
  clearLabel?: string;
  /** 내부 `<input>` 참조. */
  ref?: Ref<HTMLInputElement>;
}

/**
 * 돋보기 아이콘과 지우기 버튼이 붙은 표준 44px 검색 입력 칸.
 *
 * 검색어 입력에서는 자동 수정·맞춤법 검사를 끈다.
 */
export function SearchInput({
  label,
  value,
  onChange,
  hideLabel = false,
  clearLabel = "검색어 지우기",
  id,
  disabled = false,
  className,
  ref,
  ...rest
}: SearchInputProps) {
  const autoId = useId();
  const inputId = id ?? autoId;
  const localRef = useRef<HTMLInputElement>(null);

  function setInputRef(node: HTMLInputElement | null) {
    localRef.current = node;
    if (typeof ref === "function") {
      ref(node);
    } else if (ref && "current" in ref) {
      (ref as RefObject<HTMLInputElement | null>).current = node;
    }
  }

  function handleClear() {
    onChange("");
    localRef.current?.focus();
  }

  return (
    <div className={[styles.field, className].filter(Boolean).join(" ")}>
      <label
        htmlFor={inputId}
        className={hideLabel ? "visually-hidden" : styles.label}
      >
        {label}
      </label>
      <div className={styles.searchWrap}>
        <svg
          className={styles.searchIcon}
          viewBox="0 0 20 20"
          fill="none"
          aria-hidden="true"
        >
          <circle cx="9" cy="9" r="6" stroke="currentColor" strokeWidth="2" />
          <path
            d="M13.5 13.5L17.5 17.5"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </svg>
        <input
          {...rest}
          ref={setInputRef}
          id={inputId}
          type="search"
          value={value}
          disabled={disabled}
          autoCorrect="off"
          autoCapitalize="none"
          spellCheck={false}
          onChange={(event) => onChange(event.target.value)}
          className={`${styles.control} ${styles.searchInput}`}
        />
        {value.length > 0 && !disabled && (
          <button
            type="button"
            className={styles.clearButton}
            aria-label={clearLabel}
            onClick={handleClear}
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
}
