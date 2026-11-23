import { invoke } from "@tauri-apps/api/core";

/**
 * 폴더 고르기 창을 띄워 고른 폴더 경로를 받는다(ADR 0011). 창은 앱이 띄운다.
 *
 * 창을 닫으면 `null`. 실패하면 직원에게 보여 줄 문장을 담은 `Error`를 던진다.
 */
export async function pickFolder(): Promise<string | null> {
  try {
    return await invoke<string | null>("pick_folder");
  } catch (failure) {
    throw new Error(
      typeof failure === "string" ? failure : "폴더를 고르지 못했습니다",
      { cause: failure },
    );
  }
}
