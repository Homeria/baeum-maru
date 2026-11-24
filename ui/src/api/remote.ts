import { invoke } from "@tauri-apps/api/core";
import type { RemoteSettingsResponse } from "./generated/RemoteSettingsResponse";

/** 접속 모드(ADR 0021)에서 이 PC가 기억한 호스트 주소와 기기 증표. 앱에 묻는다. */
export function remoteSettings(): Promise<RemoteSettingsResponse> {
  return call<RemoteSettingsResponse>("remote_settings");
}

/** 호스트 주소를 기억한다. 앱이 다듬은 주소(예: `http://` 뗌)를 돌려준다. */
export function saveHostAddress(address: string): Promise<string> {
  return call<string>("save_host_address", { address });
}

/** 승인을 요청하며 받은 기기 증표와 승인 번호를 기억한다. */
export function saveDeviceRequest(token: string, code: string): Promise<void> {
  return call<void>("save_device_request", { token, code });
}

/** 기기 증표와 승인 번호를 버린다. 거절·만료되었거나 승인을 취소당했을 때. */
export function clearDeviceRequest(): Promise<void> {
  return call<void>("clear_device_request");
}

/** 승인받아 호스트에 이어졌다고 기억한다. 이 뒤로는 호스트로 바꿀 때 관리자 PIN이 필요하다. */
export function rememberApproved(): Promise<void> {
  return call<void>("remember_approved");
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
