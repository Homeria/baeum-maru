import { invoke } from "@tauri-apps/api/core";
import type { RunMode } from "./generated/RunMode";
import type { RunModeResponse } from "./generated/RunModeResponse";

/**
 * 켤 때 정한 실행 모드(ADR 0021). 아직 고르지 않았으면 `mode`가 `null`이다.
 *
 * 서버가 아니라 앱에 묻는다. 호스트로 정해지기 전에는 서버를 띄우지 않기 때문이다.
 */
export function runModeState(): Promise<RunModeResponse> {
  return invoke<RunModeResponse>("run_mode_state");
}

/** 처음 켤 때 고른 실행 모드를 기억한다. 성공하면 앱이 다시 켜져서 돌아오지 않는다. */
export async function chooseRunMode(mode: RunMode): Promise<void> {
  try {
    await invoke<void>("choose_run_mode", { mode });
  } catch (failure) {
    throw new Error(
      typeof failure === "string" ? failure : "실행 모드를 정하지 못했습니다",
      { cause: failure },
    );
  }
}
