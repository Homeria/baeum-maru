import type { ReactNode } from "react";
import styles from "./StatusPill.module.css";

/** 꼬리표 색 성격. `turn`은 선발 결과의 '연락할 차례' 전용이다. */
export type StatusPillTone = "neutral" | "primary" | "ok" | "bad" | "turn";

/** {@link StatusPill}의 속성. */
export interface StatusPillProps {
  /** 상태 색 성격. 기본값은 `neutral`. */
  tone?: StatusPillTone;
  /** 상태를 설명하는 짧은 글자. 색만으로 뜻을 전하지 않도록 반드시 글자를 넣는다. */
  children: ReactNode;
  className?: string;
}

/**
 * 상태를 글자와 테두리·바탕으로 함께 보여 주는 작은 꼬리표.
 *
 * Windows 고대비 모드에서도 구분되도록 모든 성격에 1px 테두리를 둔다.
 */
export function StatusPill({
  tone = "neutral",
  children,
  className,
}: StatusPillProps) {
  const classes = [styles.pill, styles[tone], className]
    .filter(Boolean)
    .join(" ");

  return <span className={classes}>{children}</span>;
}
