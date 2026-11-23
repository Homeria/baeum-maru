import { invoke } from "@tauri-apps/api/core";

/**
 * 파일 고르기 창을 띄워 복원할 백업 파일(`.db`) 경로를 받는다(ADR 0011). 창은 앱이 띄운다.
 *
 * 창을 닫으면 `null`. 실패하면 직원에게 보여 줄 문장을 담은 `Error`를 던진다.
 */
export async function pickBackupFile(): Promise<string | null> {
  try {
    return await invoke<string | null>("pick_backup_file");
  } catch (failure) {
    throw new Error(
      typeof failure === "string" ? failure : "파일을 고르지 못했습니다",
      { cause: failure },
    );
  }
}

/** 앱을 다시 켠다. 복원을 준비한 뒤 부른다(ADR 0011). */
export async function restartApp(): Promise<void> {
  try {
    await invoke("restart_app");
  } catch (failure) {
    throw new Error(
      "앱을 다시 켜지 못했습니다. 앱을 직접 껐다 켜면 복원됩니다",
      {
        cause: failure,
      },
    );
  }
}
