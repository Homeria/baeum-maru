import { APP_VERSION } from "../../../api/client";
import {
  StatusPill,
  type StatusPillTone,
} from "../../../components/ui/StatusPill";
import styles from "./HostStatus.module.css";
import { useHostVersion } from "./useHostVersion";

/** 사이드바 상단: 호스트 연결 상태 꼬리표와 앱·호스트 버전. */
export function HostStatus() {
  const { data, isPending, isError } = useHostVersion();

  let label: string;
  let tone: StatusPillTone;
  let versionLine: string;

  if (isPending) {
    label = "호스트 확인 중";
    tone = "neutral";
    versionLine = `앱 ${APP_VERSION}`;
  } else if (isError) {
    label = "호스트 연결 안 됨";
    tone = "bad";
    versionLine = `앱 ${APP_VERSION}`;
  } else if (!data.compatible) {
    label = "버전 불일치";
    tone = "bad";
    versionLine = `앱 ${APP_VERSION} · 호스트 ${data.hostVersion}`;
  } else {
    label = "호스트 연결됨";
    tone = "ok";
    versionLine = `앱 ${APP_VERSION} · 호스트 ${data.hostVersion}`;
  }

  return (
    <div className={styles.status}>
      <StatusPill tone={tone}>{label}</StatusPill>
      <span className={styles.version}>{versionLine}</span>
    </div>
  );
}
