import { getJson, postJson, putJson } from "./client";
import type { BackupListResponse } from "./generated/BackupListResponse";
import type { BackupResponse } from "./generated/BackupResponse";
import type { SecondLocationResponse } from "./generated/SecondLocationResponse";

/** 백업 목록, 마지막 자동 백업 실패, 두 번째 위치(ADR 0011). 관리자만. */
export function listBackups(): Promise<BackupListResponse> {
  return getJson<BackupListResponse>("/api/backups");
}

/** 지금 백업. 관리자만. */
export function createBackup(): Promise<BackupResponse> {
  return postJson<BackupResponse>("/api/backups", {});
}

/** 두 번째 위치를 `path`로 바꾼다. `null`이면 끈다. 관리자가 호스트 PC에서만. */
export function setSecondLocation(
  path: string | null,
): Promise<SecondLocationResponse> {
  return putJson<SecondLocationResponse>("/api/backups/second-location", {
    path,
  });
}
