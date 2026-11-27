import { Button } from "../ui/Button";
import styles from "./NoticeBanner.module.css";

/** 안내 배너 성격. */
export type NoticeBannerTone = "neutral" | "error";

/** {@link NoticeBanner}의 속성. */
export interface NoticeBannerProps {
  /** 상태 성격. `error`는 빨간 테두리와 `role="alert"`를 쓴다. */
  tone?: NoticeBannerTone;
  /** 강조할 짧은 제목. */
  title?: string;
  /** 안내 본문 문장. */
  message: string;
  /** 다시 해 보기 같은 동작 버튼 문구. */
  actionLabel?: string;
  /** 동작 버튼을 눌렀을 때 실행할 함수. */
  onAction?: () => void;
  className?: string;
}

/** 빈 목록이나 불러오기 실패를 카드 안에서 분명히 알리는 공통 배너. */
export function NoticeBanner({
  tone = "neutral",
  title,
  message,
  actionLabel,
  onAction,
  className,
}: NoticeBannerProps) {
  return (
    <div
      role={tone === "error" ? "alert" : "status"}
      className={[styles.banner, styles[tone], className]
        .filter(Boolean)
        .join(" ")}
    >
      <div className={styles.text}>
        {title && <p className={styles.title}>{title}</p>}
        <p className={styles.message}>{message}</p>
      </div>
      {actionLabel && onAction && (
        <Button size="sm" variant="secondary" onClick={onAction}>
          {actionLabel}
        </Button>
      )}
    </div>
  );
}
