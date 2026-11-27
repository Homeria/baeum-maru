import { useId, useRef, useState, type ReactNode } from "react";
import { BottomAlertStack, type BottomAlertItem } from "./BottomAlertStack";
import styles from "./AppShell.module.css";

function SidebarToggleIcon({
  direction,
}: {
  direction: "collapse" | "expand";
}) {
  return (
    <svg
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      aria-hidden="true"
    >
      <rect
        x="2.25"
        y="3.25"
        width="15.5"
        height="13.5"
        rx="2.25"
        stroke="currentColor"
        strokeWidth="1.6"
      />
      <line
        x1="7.25"
        y1="3.5"
        x2="7.25"
        y2="16.5"
        stroke="currentColor"
        strokeWidth="1.6"
      />
      {direction === "collapse" ? (
        <path
          d="M13.25 7.5L10.75 10L13.25 12.5"
          stroke="currentColor"
          strokeWidth="1.6"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      ) : (
        <path
          d="M11 7.5L13.5 10L11 12.5"
          stroke="currentColor"
          strokeWidth="1.6"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
    </svg>
  );
}

/** 왼쪽 메뉴 항목 하나. */
export interface AppShellNavItem<K extends string> {
  key: K;
  label: string;
  /** 항목 오른쪽에 덧붙일 짧은 배지(예: 승인 대기 건수). */
  badge?: ReactNode;
}

/** 구분선으로 나뉘는 왼쪽 메뉴 묶음(예: 수강 사무 / 기관 운영). */
export interface AppShellNavSection<K extends string> {
  label: string;
  items: readonly AppShellNavItem<K>[];
}

/** {@link AppShell}의 속성. */
export interface AppShellProps<K extends string> {
  /** 메뉴 묶음 목록. */
  sections: readonly AppShellNavSection<K>[];
  /** 현재 열린 메뉴 키. */
  selected: K;
  /** 메뉴를 골랐을 때 호출된다. */
  onSelect: (key: K) => void;
  /** 오른쪽 작업 영역 머리줄의 제목(`<h1>`). */
  pageTitle: string;
  /** 제목 옆에 붙일 상태 꼬리표(예: 조회 전용 안내). */
  headerStatus?: ReactNode;
  /** 머리줄 오른쪽에 둘 주 버튼이나 작은 탭. */
  headerActions?: ReactNode;
  /** 왼쪽 메뉴 맨 위 브랜드 아래에 둘 호스트 연결 상태. */
  hostStatus?: ReactNode;
  /** 왼쪽 메뉴 맨 아래에 둘 로그인 직원·잠금·로그아웃 영역. */
  sidebarFooter?: ReactNode;
  /** 화면 맨 아래에 쌓일 상태 알림 목록. */
  alerts?: readonly BottomAlertItem[];
  /** 화면 잠금 여부. 참이면 뒤쪽 껍데기 전체를 `inert`로 막는다. */
  locked?: boolean;
  /** 초기 메뉴 접힘 여부(비제어). 기본값은 `false`. */
  defaultCollapsed?: boolean;
  /** 외부에서 메뉴 접힘 상태를 제어할 때 넘긴다. */
  collapsed?: boolean;
  /** 메뉴 접힘 상태가 바뀔 때 호출된다. */
  onCollapsedChange?: (collapsed: boolean) => void;
  children: ReactNode;
}

/**
 * 로그인 뒤 메인 화면의 공통 껍데기.
 *
 * 왼쪽 남색 메뉴는 접기/펼치기를 지원하며, 접으면 오른쪽 작업 영역이 가로 전체를 쓰고
 * 머리줄 맨 왼쪽에 "메뉴 열기" 버튼이 나타난다. 하단 알림 띠는 본문 아래에 고정되어
 * 작업 영역을 위로 밀어 주 버튼을 가리지 않는다.
 */
export function AppShell<K extends string>({
  sections,
  selected,
  onSelect,
  pageTitle,
  headerStatus,
  headerActions,
  hostStatus,
  sidebarFooter,
  alerts = [],
  locked = false,
  defaultCollapsed = false,
  collapsed: controlledCollapsed,
  onCollapsedChange,
  children,
}: AppShellProps<K>) {
  const [uncontrolledCollapsed, setUncontrolledCollapsed] =
    useState(defaultCollapsed);
  const navId = useId();
  const collapseButtonRef = useRef<HTMLButtonElement>(null);
  const expandButtonRef = useRef<HTMLButtonElement>(null);

  const isCollapsed = controlledCollapsed ?? uncontrolledCollapsed;

  function setCollapsed(next: boolean) {
    if (controlledCollapsed === undefined) {
      setUncontrolledCollapsed(next);
    }
    onCollapsedChange?.(next);
    // 버튼이 바뀐 뒤 키보드 초점이 유실되지 않도록 대응 버튼으로 옮긴다
    queueMicrotask(() => {
      if (next) {
        expandButtonRef.current?.focus();
      } else {
        collapseButtonRef.current?.focus();
      }
    });
  }

  return (
    <div
      className={[styles.shell, isCollapsed ? styles.collapsed : ""]
        .filter(Boolean)
        .join(" ")}
      inert={locked}
    >
      {!isCollapsed && (
        <aside className={styles.sidebar} aria-label="주 메뉴">
          <div className={styles.sidebarTop}>
            <div className={styles.brandRow}>
              <span className={styles.brand}>배움마루</span>
              <button
                ref={collapseButtonRef}
                type="button"
                className={styles.collapseButton}
                aria-label="메뉴 접기"
                title="메뉴 접기"
                aria-expanded="true"
                aria-controls={navId}
                onClick={() => setCollapsed(true)}
              >
                <SidebarToggleIcon direction="collapse" />
              </button>
            </div>
            {hostStatus && (
              <div className={styles.hostStatus}>{hostStatus}</div>
            )}
          </div>

          <nav id={navId} aria-label="업무 메뉴" className={styles.nav}>
            {sections.map((section, sectionIndex) => (
              <div
                key={section.label}
                className={[
                  styles.section,
                  sectionIndex > 0 ? styles.sectionDivided : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
              >
                <p className={styles.sectionLabel}>{section.label}</p>
                <ul className={styles.navList}>
                  {section.items.map((item) => {
                    const active = item.key === selected;
                    return (
                      <li key={item.key}>
                        <button
                          type="button"
                          aria-current={active ? "page" : undefined}
                          className={[
                            styles.navButton,
                            active ? styles.navActive : "",
                          ]
                            .filter(Boolean)
                            .join(" ")}
                          onClick={() => onSelect(item.key)}
                        >
                          <span className={styles.navLabel}>{item.label}</span>
                          {item.badge && (
                            <span className={styles.navBadge}>
                              {item.badge}
                            </span>
                          )}
                        </button>
                      </li>
                    );
                  })}
                </ul>
              </div>
            ))}
          </nav>

          {sidebarFooter && (
            <div className={styles.sidebarFooter}>{sidebarFooter}</div>
          )}
        </aside>
      )}

      <div className={styles.workspace}>
        <header className={styles.header}>
          <div className={styles.headerLeft}>
            {isCollapsed && (
              <button
                ref={expandButtonRef}
                type="button"
                className={styles.expandButton}
                aria-label="메뉴 열기"
                title="메뉴 열기"
                aria-expanded="false"
                aria-controls={navId}
                onClick={() => setCollapsed(false)}
              >
                <SidebarToggleIcon direction="expand" />
              </button>
            )}
            <h1 className={styles.pageTitle}>{pageTitle}</h1>
            {headerStatus && (
              <div className={styles.headerStatus}>{headerStatus}</div>
            )}
          </div>
          {headerActions && (
            <div className={styles.headerActions}>{headerActions}</div>
          )}
        </header>

        <main className={styles.main}>{children}</main>

        <BottomAlertStack items={alerts} />
      </div>
    </div>
  );
}
