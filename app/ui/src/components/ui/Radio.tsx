import { useId, type InputHTMLAttributes, type Ref } from "react";
import styles from "./field.module.css";

/** {@link Radio}의 속성. */
export interface RadioProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "type" | "children"
> {
  /** 라디오 버튼 옆에 보여 줄 이름. */
  label: string;
  /** 이름 아래에 덧붙일 보조 설명. `aria-describedby`로 연결된다. */
  description?: string;
  /** 내부 `<input type="radio">` 참조. */
  ref?: Ref<HTMLInputElement>;
}

/** 20px 크기와 넉넉한 클릭 영역을 갖춘 공통 라디오 버튼. */
export function Radio({
  label,
  description,
  id,
  className,
  ref,
  "aria-describedby": describedByProp,
  ...rest
}: RadioProps) {
  const autoId = useId();
  const inputId = id ?? autoId;
  const descId = description ? `${inputId}-desc` : undefined;
  const describedBy =
    [describedByProp, descId].filter(Boolean).join(" ") || undefined;

  return (
    <div className={[styles.choiceRow, className].filter(Boolean).join(" ")}>
      <input
        {...rest}
        ref={ref}
        id={inputId}
        type="radio"
        aria-describedby={describedBy}
        className={styles.choiceInput}
      />
      <div className={styles.choiceText}>
        <label htmlFor={inputId} className={styles.choiceLabel}>
          {label}
        </label>
        {description && (
          <p id={descId} className={styles.hint}>
            {description}
          </p>
        )}
      </div>
    </div>
  );
}
