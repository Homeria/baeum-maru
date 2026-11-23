import { getJson, postJson, postNoContent, putJson } from "./client";
import type { BackupListResponse } from "./generated/BackupListResponse";
import type { BackupResponse } from "./generated/BackupResponse";
import type { RestorePreviewResponse } from "./generated/RestorePreviewResponse";
import type { RestoreRequest } from "./generated/RestoreRequest";
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

/** 복원할 백업. 목록의 파일 이름이나 다른 곳에서 고른 파일의 경로(호스트 PC의 경로). */
export type RestoreSource = { fileName: string } | { path: string };

function restoreRequest(source: RestoreSource): RestoreRequest {
  return "fileName" in source
    ? { fileName: source.fileName, path: null }
    : { fileName: null, path: source.path };
}

/** 복원할 백업의 미리보기. 관리자가 호스트 PC에서만. */
export function previewRestore(
  source: RestoreSource,
): Promise<RestorePreviewResponse> {
  return postJson<RestorePreviewResponse>(
    "/api/backups/restore-preview",
    restoreRequest(source),
  );
}

/**
 * 복원을 준비한다. 지금 상태를 복원 직전 백업으로 남긴다. 앱을 다시 켜면 바뀐다.
 * 관리자가 호스트 PC에서만.
 */
export function prepareRestore(source: RestoreSource): Promise<void> {
  return postNoContent("/api/backups/restore", restoreRequest(source));
}
