import type { ReactNode } from "react";
import styles from "./SplitAuthLayout.module.css";

/** 좌측 남색 안내 패널에 표시할 진행 단계 하나. */
export interface SplitAuthStep {
  /** 단계 이름(예: "관리자 계정 만들기"). */
  label: string;
  /** 단계 아래 짧은 설명. 선택. */
  description?: string;
  /** 단계 상태. */
  status: "done" | "current" | "upcoming";
}

/** {@link SplitAuthLayout}의 속성. */
export interface SplitAuthLayoutProps {
  /** 좌측 남색 패널 제목 위 작은 꼬리표(예: "선택 단계"). */
  panelBadge?: string;
  /** 좌측 남색 패널 제목. */
  panelTitle: string;
  /** 좌측 남색 패널 설명 문장. */
  panelDescription?: string;
  /** 브랜드 아래 보조 문구. 기본값은 "소규모 기관 수강 사무 프로그램". */
  brandSubtitle?: string;
  /** 처음 설정처럼 여러 단계를 거칠 때 표시할 단계 목록. */
  steps?: readonly SplitAuthStep[];
  /** 좌측 남색 패널 맨 아래에 둘 보조 영역. */
  panelFooter?: ReactNode;
  /** 오른쪽 흰 카드 제목(`<h1>`). `frameless` 모드에서는 생략할 수 있다. */
  cardTitle?: string;
  /** 오른쪽 흰 카드 제목 아래 보조 설명. */
  cardSubtitle?: string;
  /** 카드 넓이. 기본 입력은 `"md"`, 넓은 단일 카드는 `"lg"`. */
  cardWidth?: "md" | "lg";
  /** 오른쪽 영역을 단일 흰 카드로 감싸지 않고 넓은 다중 카드 작업 공간으로 쓴다. */
  frameless?: boolean;
  /** 카드 아래쪽에 둘 보조 링크나 안내. */
  cardFooter?: ReactNode;
  children: ReactNode;
}

/**
 * 로그인 전·특수 흐름(실행 모드, 접속, 처음 설정, 로그인, 잠금, 복원, 넘기기) 공통 좌우 분할 틀.
 *
 * 왼쪽 남색 패널은 현재 흐름과 단계를 보여 주고, 오른쪽 흰 카드(또는 다중 카드 영역)가 실제 입력과 선택을 담는다.
 */
export function SplitAuthLayout({
  panelBadge,
  panelTitle,
  panelDescription,
  brandSubtitle = "소규모 기관 수강 사무 프로그램",
  steps,
  panelFooter,
  cardTitle,
  cardSubtitle,
  cardWidth = "md",
  frameless = false,
  cardFooter,
  children,
}: SplitAuthLayoutProps) {
  const HeadingTag = cardTitle ? "h2" : "h1";

  return (
    <div className={styles.layout}>
      <aside className={styles.panel} aria-label="안내">
        <div className={styles.brand}>
          <p className={styles.brandName}>배움마루</p>
          <p className={styles.brandSubtitle}>{brandSubtitle}</p>
        </div>

        <div className={styles.panelBody}>
          {panelBadge && (
            <span className={styles.panelBadge}>{panelBadge}</span>
          )}
          <HeadingTag className={styles.panelTitle}>{panelTitle}</HeadingTag>
          {panelDescription && (
            <p className={styles.panelDescription}>{panelDescription}</p>
          )}
          {steps && steps.length > 0 && (
            <nav aria-label="진행 단계" className={styles.stepsNav}>
              <ol className={styles.stepsList}>
                {steps.map((step, index) => (
                  <li
                    key={step.label}
                    aria-current={
                      step.status === "current" ? "step" : undefined
                    }
                    className={[styles.stepItem, styles[step.status]].join(" ")}
                  >
                    <span className={styles.stepNumber} aria-hidden="true">
                      {index + 1}
                    </span>
                    <div className={styles.stepText}>
                      <span className={styles.stepLabel}>{step.label}</span>
                      {step.description && (
                        <span className={styles.stepDescription}>
                          {step.description}
                        </span>
                      )}
                    </div>
                  </li>
                ))}
              </ol>
            </nav>
          )}
        </div>

        {panelFooter && <div className={styles.panelFooter}>{panelFooter}</div>}
      </aside>

      <main
        className={[styles.main, frameless ? styles.mainFrameless : ""]
          .filter(Boolean)
          .join(" ")}
      >
        {frameless ? (
          <div className={styles.workspace}>{children}</div>
        ) : (
          <div
            className={[
              styles.card,
              cardWidth === "lg" ? styles.cardLg : styles.cardMd,
            ].join(" ")}
          >
            {cardTitle && (
              <header className={styles.cardHeader}>
                <h1 className={styles.cardTitle}>{cardTitle}</h1>
                {cardSubtitle && (
                  <p className={styles.cardSubtitle}>{cardSubtitle}</p>
                )}
              </header>
            )}
            <div className={styles.cardBody}>{children}</div>
            {cardFooter && (
              <div className={styles.cardFooter}>{cardFooter}</div>
            )}
          </div>
        )}
      </main>
    </div>
  );
}
