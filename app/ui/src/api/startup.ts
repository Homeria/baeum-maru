import { invoke } from "@tauri-apps/api/core";
import type { RestoreSource } from "./backups";
import type { RestorePreviewResponse } from "./generated/RestorePreviewResponse";
import type { RestoreRequest } from "./generated/RestoreRequest";
import type { StartupStateResponse } from "./generated/StartupStateResponse";
import { invokeCommand } from "./appCommands";

/**
 * 켤 때 이 PC의 DB 상태(ADR 0011). 깨졌으면 백업 목록과 권하는 백업도 준다.
 *
 * 서버가 아니라 앱에 묻는다. DB가 깨지면 서버를 띄우지 않기 때문이다.
 */
export function startupState(): Promise<StartupStateResponse> {
  return invoke<StartupStateResponse>("startup_state");
}

/** DB가 깨졌을 때 고른 백업의 미리보기. */
export function previewBrokenDbBackup(
  source: RestoreSource,
): Promise<RestorePreviewResponse> {
  return call<RestorePreviewResponse>("preview_broken_db_backup", source);
}

/** 깨진 DB를 고른 백업으로 되돌린다. 성공하면 앱이 다시 켜져서 돌아오지 않는다. */
export function restoreBrokenDb(source: RestoreSource): Promise<void> {
  return call<void>("restore_broken_db", source);
}

/** 복원할 파일을 요청 모양으로 바꿔 앱 명령을 부른다. */
function call<T>(command: string, source: RestoreSource): Promise<T> {
  const request: RestoreRequest =
    "fileName" in source
      ? { fileName: source.fileName, path: null }
      : { fileName: null, path: source.path };
  return invokeCommand<T>(command, { request });
}
