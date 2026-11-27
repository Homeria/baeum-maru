import type { ReactNode } from "react";
import styles from "./Card.module.css";

/** {@link Card}의 속성. */
export interface CardProps {
  /** 카드 위쪽 머리줄에 보여 줄 제목. */
  title?: string;
  /** 제목 수준. 기본값은 `2`(`<h2>`). */
  headingLevel?: 2 | 3;
  /** 제목 아래에 덧붙일 짧은 설명. */
  subtitle?: string;
  /** 머리줄 오른쪽에 배치할 버튼이나 꼬리표. */
  actions?: ReactNode;
  /** 본문 안쪽 여백. 표가 카드 끝까지 닿을 때는 `"none"`을 쓴다. */
  padding?: "md" | "none";
  children: ReactNode;
  className?: string;
}

/** 흰 바탕과 1px 테두리로 화면 영역을 구분하는 공통 카드. */
export function Card({
  title,
  headingLevel = 2,
  subtitle,
  actions,
  padding = "md",
  children,
  className,
}: CardProps) {
  const hasHeader = Boolean(title || subtitle || actions);
  const Heading = headingLevel === 3 ? "h3" : "h2";

  return (
    <section className={[styles.card, className].filter(Boolean).join(" ")}>
      {hasHeader && (
        <div className={styles.header}>
          <div className={styles.headingGroup}>
            {title && <Heading className={styles.title}>{title}</Heading>}
            {subtitle && <p className={styles.subtitle}>{subtitle}</p>}
          </div>
          {actions && <div className={styles.actions}>{actions}</div>}
        </div>
      )}
      <div className={padding === "md" ? styles.body : styles.bodyFlush}>
        {children}
      </div>
    </section>
  );
}
