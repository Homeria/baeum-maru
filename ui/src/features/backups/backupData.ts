import type { BackupKindCode } from "../../api/generated/BackupKindCode";
import type { BackupResponse } from "../../api/generated/BackupResponse";
import type { SecondLocationResponse } from "../../api/generated/SecondLocationResponse";

/** 백업 종류의 화면 이름. 용어집의 "백업 종류". */
export const BACKUP_KIND_LABELS: Record<BackupKindCode, string> = {
  daily: "하루 한 번",
  manual: "지금 백업",
  exit: "앱 종료",
  before_lottery: "추첨 직전",
  before_close: "선발 마감 직전",
  before_migration: "구조 변경 직전",
  before_restore: "복원 직전",
};

/**
 * 두 번째 위치에 이만큼 넘게 복사하지 못했으면 경고한다. 주말에 앱을 켜지 않는 기관이
 * 많아서 하루 이틀 비는 것은 경고하지 않는다.
 */
export const STALE_DAYS = 3;

/** 목록의 종류 칸. 예: `추첨 직전 · 요가 초급` */
export function kindText(
  backup: Pick<BackupResponse, "kind" | "courseId" | "courseName">,
): string {
  const label = BACKUP_KIND_LABELS[backup.kind];
  if (backup.courseId === null) {
    return label;
  }
  return `${label} · ${backup.courseName ?? `지운 강좌(${backup.courseId})`}`;
}

/** 호스트 지역 시각 `2026-11-23T09:05:00`을 `2026-11-23 09:05`로. */
export function formatTakenAt(local: string): string {
  return local.replace("T", " ").slice(0, 16);
}

/** 바이트를 읽기 쉬운 크기로. 예: `1.5 MB` */
export function formatSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/**
 * 두 번째 위치에 대한 경고 문장. 꺼져 있거나 문제가 없으면 `null`.
 *
 * `today`는 이 PC의 오늘 날짜다. 시각은 호스트 지역 시각이라 날짜만 비교한다.
 */
export function secondLocationWarning(
  second: SecondLocationResponse,
  today: Date,
): string | null {
  if (second.path === null) {
    return null;
  }
  if (!second.reachable) {
    return "두 번째 위치 폴더에 연결되지 않았습니다. 외장 디스크나 USB가 꽂혀 있는지 확인해 주세요";
  }
  if (second.lastCopiedAt === null) {
    return "두 번째 위치에 아직 복사한 백업이 없습니다";
  }
  const days = daysBetween(second.lastCopiedAt.slice(0, 10), today);
  if (days > STALE_DAYS) {
    return `두 번째 위치에 ${days}일째 복사하지 못했습니다`;
  }
  return null;
}

/** `YYYY-MM-DD`부터 `today`까지 며칠인지. */
function daysBetween(date: string, today: Date): number {
  const [year, month, day] = date.split("-").map(Number);
  const from = Date.UTC(year, month - 1, day);
  const to = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
  return Math.round((to - from) / (24 * 60 * 60 * 1000));
}
