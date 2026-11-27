import type { ButtonHTMLAttributes, Ref } from "react";
import styles from "./Button.module.css";

/** 버튼 성격. */
export type ButtonVariant = "primary" | "secondary" | "danger" | "dangerSolid";

/** 버튼 높이 단계. `md`는 표준 44px, `sm`은 표·줄 동작 32px. */
export type ButtonSize = "md" | "sm";

/** {@link Button}의 속성. */
export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** 시각적 강조 수준. 기본값은 `secondary`. */
  variant?: ButtonVariant;
  /** 높이 단계. 기본값은 `md`. */
  size?: ButtonSize;
  /** 요청 처리 중 여부. 참이면 중복 누름을 막고 `aria-busy`를 붙인다. */
  busy?: boolean;
  /** `busy`일 때 대신 보여 줄 문구. */
  busyLabel?: string;
  /** 내부 `<button>` 참조. */
  ref?: Ref<HTMLButtonElement>;
}

/**
 * 화면 전체에서 쓰는 공통 버튼.
 *
 * 기본 `type`을 `"button"`으로 두어 보조 버튼이 실수로 폼을 제출하지 않게 한다.
 */
export function Button({
  variant = "secondary",
  size = "md",
  busy = false,
  busyLabel,
  type = "button",
  disabled = false,
  className,
  children,
  ref,
  ...rest
}: ButtonProps) {
  const classes = [styles.button, styles[variant], styles[size], className]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      {...rest}
      ref={ref}
      type={type}
      disabled={disabled || busy}
      aria-busy={busy || undefined}
      className={classes}
    >
      {busy && busyLabel ? busyLabel : children}
    </button>
  );
}
