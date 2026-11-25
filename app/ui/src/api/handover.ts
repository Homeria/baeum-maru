import { postNoContent } from "./client";
import type { HandedOverAdmin } from "./generated/HandedOverAdmin";
import type { HandoverFileResponse } from "./generated/HandoverFileResponse";
import type { TakeOverPreviewResponse } from "./generated/TakeOverPreviewResponse";
import { invokeCommand } from "./appCommands";

/**
 * BR-H1: 로그인한 관리자의 PIN으로 호스트 넘기기를 시작한다. 호스트 PC에서만.
 * 성공하면 화면이 앱을 다시 켜고, 서버 없이 넘기기 파일을 만든다.
 */
export function startHandover(pin: string): Promise<void> {
  return postNoContent("/api/handover", { pin });
}

/** 넘기는 중에 켜면서 만든 넘기기 파일의 미리보기. 앱에 묻는다. */
export function handoverFile(): Promise<HandoverFileResponse> {
  return invokeCommand<HandoverFileResponse>("handover_file");
}

/**
 * 저장 창에서 고른 곳에 넘기기 파일을 저장한다. 성공하면 이 PC가 "넘김"이 되고 앱이 다시 켜져서
 * 돌아오지 않는다. 저장 창을 닫으면 `false`.
 */
export function saveHandoverFile(): Promise<boolean> {
  return invokeCommand<boolean>("save_handover_file");
}

/** 넘기지 않고 다시 호스트로 켠다. 성공하면 돌아오지 않는다. */
export function abortHandover(): Promise<void> {
  return invokeCommand<void>("abort_handover");
}

/** 넘긴 PC에서 넘기기를 취소할 수 있는 관리자들(BR-H2). */
export function handedOverAdmins(): Promise<HandedOverAdmin[]> {
  return invokeCommand<HandedOverAdmin[]>("handed_over_admins");
}

/** BR-H2: 관리자 PIN으로 넘기기를 취소하고 다시 호스트로 켠다. 성공하면 돌아오지 않는다. */
export function cancelHandover(staffId: number, pin: string): Promise<void> {
  return invokeCommand<void>("cancel_handover", { staffId, pin });
}

/** 파일 고르기 창으로 넘겨받을 넘기기 파일(`.db`)을 고른다(BR-H3). 창을 닫으면 `null`. */
export function pickHandoverFile(): Promise<string | null> {
  return invokeCommand<string | null>("pick_handover_file");
}

/** 고른 넘기기 파일의 미리보기와 확인에 쓸 파일 안 관리자들. */
export function previewTakeOver(
  path: string,
): Promise<TakeOverPreviewResponse> {
  return invokeCommand<TakeOverPreviewResponse>("preview_take_over", { path });
}

/**
 * BR-H3: 파일 안 관리자의 PIN으로 확인하고 이 PC를 새 호스트로 시작한다.
 * 성공하면 앱이 다시 켜져서 돌아오지 않는다.
 */
export function takeOver(
  path: string,
  staffId: number,
  pin: string,
): Promise<void> {
  return invokeCommand<void>("take_over", { path, staffId, pin });
}
