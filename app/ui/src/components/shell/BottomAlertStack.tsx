import { Button } from "../ui/Button";
import styles from "./BottomAlertStack.module.css";

/** 하단 알림 띠의 성격. 노랑은 '연락할 차례' 전용이라 여기서는 쓰지 않는다. */
export type BottomAlertTone = "danger" | "info" | "neutral";

/** {@link BottomAlertStack}에 표시할 알림 항목 하나. */
export interface BottomAlertItem {
  /** 항목 고유 키. */
  id: string;
  /** 색과 화면 읽기 프로그램 알림 수준. `danger`는 `alert`, 나머지는 `status`. */
  tone: BottomAlertTone;
  /** 직원에게 보여 줄 상태 문구. */
  message: string;
  /** 확인·이동 버튼 문구(예: "확인하기"). */
  actionLabel?: string;
  /** 확인·이동 버튼을 눌렀을 때 실행할 함수. */
  onAction?: () => void;
  /** 닫기 버튼 접근성 이름. 기본값은 "닫기". */
  dismissLabel?: string;
  /** 닫을 수 있는 알림일 때만 넘긴다. 지속 상태(연결 끊김 등)는 비워 둔다. */
  onDismiss?: () => void;
}

/** {@link BottomAlertStack}의 속성. */
export interface BottomAlertStackProps {
  items: readonly BottomAlertItem[];
  className?: string;
}

/**
 * 화면 맨 아래에 차곡차곡 쌓이는 상태 알림 띠.
 *
 * 작업 영역을 덮지 않고 위로 밀어 올려 하단 주 버튼(신청하기 등)을 가리지 않는다.
 * 화면 읽기 프로그램이 새 알림을 놓치지 않도록 빈 상태에서도 `alert`·`status` 라이브 영역을 유지한다.
 */
export function BottomAlertStack({ items, className }: BottomAlertStackProps) {
  const urgentItems = items.filter((item) => item.tone === "danger");
  const politeItems = items.filter((item) => item.tone !== "danger");

  return (
    <div
      role="region"
      aria-label="상태 알림"
      className={[
        styles.stack,
        items.length === 0 ? styles.empty : "",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
    >
      <div role="alert" aria-atomic="false" className={styles.group}>
        {urgentItems.map((item) => (
          <AlertRow key={item.id} item={item} />
        ))}
      </div>
      <div role="status" aria-atomic="false" className={styles.group}>
        {politeItems.map((item) => (
          <AlertRow key={item.id} item={item} />
        ))}
      </div>
    </div>
  );
}

function AlertRow({ item }: { item: BottomAlertItem }) {
  return (
    <div className={[styles.item, styles[item.tone]].join(" ")}>
      <p className={styles.message}>{item.message}</p>
      {(item.actionLabel || item.onDismiss) && (
        <div className={styles.actions}>
          {item.actionLabel && item.onAction && (
            <Button
              size="sm"
              variant={item.tone === "info" ? "primary" : "secondary"}
              onClick={item.onAction}
            >
              {item.actionLabel}
            </Button>
          )}
          {item.onDismiss && (
            <Button size="sm" variant="secondary" onClick={item.onDismiss}>
              {item.dismissLabel ?? "닫기"}
            </Button>
          )}
        </div>
      )}
    </div>
  );
}
