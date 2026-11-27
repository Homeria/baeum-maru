import type { ReactNode } from "react";
import styles from "./SplitAuthLayout.module.css";

/** 좌측 남색 안내 패널에 표시할 진행 단계 하나. */
export interface SplitAuthStep {
  /** 단계 이름(예: "관리자 계정 만들기"). */
  label: string;
  /** 단계 상태. */
  status: "done" | "current" | "upcoming";
}

/** {@link SplitAuthLayout}의 속성. */
export interface SplitAuthLayoutProps {
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
  /** 오른쪽 흰 카드 제목(`<h1>`). */
  cardTitle: string;
  /** 오른쪽 흰 카드 제목 아래 보조 설명. */
  cardSubtitle?: string;
  /** 카드 넓이. 기본 입력은 `"md"`, 기초 정보 미리 넣기처럼 넓은 폼은 `"lg"`. */
  cardWidth?: "md" | "lg";
  /** 카드 아래쪽에 둘 보조 링크나 안내. */
  cardFooter?: ReactNode;
  children: ReactNode;
}

/**
 * 로그인 전·특수 흐름(실행 모드, 접속, 처음 설정, 로그인, 잠금, 복원, 넘기기) 공통 좌우 분할 틀.
 *
 * 왼쪽 남색 패널은 현재 흐름과 단계를 보여 주고, 오른쪽 흰 카드가 실제 입력과 선택을 담는다.
 */
export function SplitAuthLayout({
  panelTitle,
  panelDescription,
  brandSubtitle = "소규모 기관 수강 사무 프로그램",
  steps,
  panelFooter,
  cardTitle,
  cardSubtitle,
  cardWidth = "md",
  cardFooter,
  children,
}: SplitAuthLayoutProps) {
  return (
    <div className={styles.layout}>
      <aside className={styles.panel} aria-label="안내">
        <div className={styles.brand}>
          <p className={styles.brandName}>배움마루</p>
          <p className={styles.brandSubtitle}>{brandSubtitle}</p>
        </div>

        <div className={styles.panelBody}>
          <h2 className={styles.panelTitle}>{panelTitle}</h2>
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
                    <span className={styles.stepLabel}>{step.label}</span>
                  </li>
                ))}
              </ol>
            </nav>
          )}
        </div>

        {panelFooter && <div className={styles.panelFooter}>{panelFooter}</div>}
      </aside>

      <main className={styles.main}>
        <div
          className={[
            styles.card,
            cardWidth === "lg" ? styles.cardLg : styles.cardMd,
          ].join(" ")}
        >
          <header className={styles.cardHeader}>
            <h1 className={styles.cardTitle}>{cardTitle}</h1>
            {cardSubtitle && (
              <p className={styles.cardSubtitle}>{cardSubtitle}</p>
            )}
          </header>
          <div className={styles.cardBody}>{children}</div>
          {cardFooter && <div className={styles.cardFooter}>{cardFooter}</div>}
        </div>
      </main>
    </div>
  );
}
