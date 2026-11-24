import styles from "./changes.module.css";

/** {@link ConnectionLostBar}의 속성. */
export interface ConnectionLostBarProps {
  lost: boolean;
}

/** 변경 알림 연결이 끊겼을 때 머리 아래에 띄우는 줄. 다시 붙으면 사라진다. */
export function ConnectionLostBar({ lost }: ConnectionLostBarProps) {
  if (!lost) {
    return null;
  }
  return (
    <div className={styles.bar} role="status">
      호스트와 연결이 끊겼습니다. 다시 연결하는 중…
      <span className={styles.hint}>
        그동안 다른 PC가 바꾼 내용은 화면에 보이지 않습니다.
      </span>
    </div>
  );
}
