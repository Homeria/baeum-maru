import { APP_VERSION } from "../../../api/client";
import { useHostVersion } from "./useHostVersion";
import styles from "./HostStatus.module.css";

/** 앱 버전과 호스트 연결 상태. */
export function HostStatus() {
  const { data, isPending, isError } = useHostVersion();

  let host: string;
  let tone: string;
  if (isPending) {
    host = "호스트 확인 중";
    tone = styles.muted;
  } else if (isError) {
    host = "호스트에 연결할 수 없습니다";
    tone = styles.bad;
  } else if (!data.compatible) {
    host = `호스트 ${data.hostVersion}: 버전이 맞지 않습니다`;
    tone = styles.bad;
  } else {
    host = `호스트 ${data.hostVersion}`;
    tone = styles.ok;
  }

  return (
    <div className={styles.status}>
      <span>앱 {APP_VERSION}</span>
      <span className={tone}>{host}</span>
    </div>
  );
}
