import { invoke } from "@tauri-apps/api/core";
import { postNoContent } from "./client";
import type { HandedOverAdmin } from "./generated/HandedOverAdmin";
import type { HandoverFileResponse } from "./generated/HandoverFileResponse";

/**
 * BR-H1: 로그인한 관리자의 PIN으로 호스트 넘기기를 시작한다. 호스트 PC에서만.
 * 성공하면 화면이 앱을 다시 켜고, 서버 없이 넘기기 파일을 만든다.
 */
export function startHandover(pin: string): Promise<void> {
  return postNoContent("/api/handover", { pin });
}

/** 넘기는 중에 켜면서 만든 넘기기 파일의 미리보기. 앱에 묻는다. */
export function handoverFile(): Promise<HandoverFileResponse> {
  return call<HandoverFileResponse>("handover_file");
}

/**
 * 저장 창에서 고른 곳에 넘기기 파일을 저장한다. 성공하면 이 PC가 "넘김"이 되고 앱이 다시 켜져서
 * 돌아오지 않는다. 저장 창을 닫으면 `false`.
 */
export function saveHandoverFile(): Promise<boolean> {
  return call<boolean>("save_handover_file");
}

/** 넘기지 않고 다시 호스트로 켠다. 성공하면 돌아오지 않는다. */
export function abortHandover(): Promise<void> {
  return call<void>("abort_handover");
}

/** 넘긴 PC에서 넘기기를 취소할 수 있는 관리자들(BR-H2). */
export function handedOverAdmins(): Promise<HandedOverAdmin[]> {
  return call<HandedOverAdmin[]>("handed_over_admins");
}

/** BR-H2: 관리자 PIN으로 넘기기를 취소하고 다시 호스트로 켠다. 성공하면 돌아오지 않는다. */
export function cancelHandover(staffId: number, pin: string): Promise<void> {
  return call<void>("cancel_handover", { staffId, pin });
}

/** 앱 명령을 부른다. 앱이 보낸 실패 문장은 그대로 `Error`로 던진다. */
async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (failure) {
    throw new Error(
      typeof failure === "string" ? failure : "처리하지 못했습니다",
      { cause: failure },
    );
  }
}
