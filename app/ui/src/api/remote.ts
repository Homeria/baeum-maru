import type { HostCheckResponse } from "./generated/HostCheckResponse";
import type { RemoteSettingsResponse } from "./generated/RemoteSettingsResponse";
import { invokeCommand } from "./appCommands";

/** 접속 모드(ADR 0021)에서 이 PC가 기억한 호스트 주소와 기기 증표. 앱에 묻는다. */
export function remoteSettings(): Promise<RemoteSettingsResponse> {
  return invokeCommand<RemoteSettingsResponse>("remote_settings");
}

/** 호스트 주소를 기억한다. 앱이 다듬은 주소(예: `http://` 뗌)를 돌려준다. */
export function saveHostAddress(address: string): Promise<string> {
  return invokeCommand<string>("save_host_address", { address });
}

/** 승인을 요청하며 받은 기기 증표와 승인 번호를 기억한다. */
export function saveDeviceRequest(token: string, code: string): Promise<void> {
  return invokeCommand<void>("save_device_request", { token, code });
}

/** 기기 증표와 승인 번호를 버린다. 거절·만료되었거나 승인을 취소당했을 때. */
export function clearDeviceRequest(): Promise<void> {
  return invokeCommand<void>("clear_device_request");
}

/** 승인받아 호스트에 이어졌다고 기억한다. 이 뒤로는 호스트로 바꿀 때 관리자 PIN이 필요하다. */
export function rememberApproved(): Promise<void> {
  return invokeCommand<void>("remember_approved");
}

/** 만난 호스트를 이 PC가 기억한 호스트와 견준다(BR-H5). 기억하지는 않는다. */
export function checkHostIdentity(
  bundleId: string,
  generation: number,
): Promise<HostCheckResponse> {
  return invokeCommand<HostCheckResponse>("check_host_identity", {
    bundleId,
    generation,
  });
}

/** 승인받아 이어진 호스트를 기억한다(BR-H5). 붙지 않을 호스트면 앱이 거절한다. */
export function rememberHostIdentity(
  bundleId: string,
  generation: number,
): Promise<void> {
  return invokeCommand<void>("remember_host_identity", {
    bundleId,
    generation,
  });
}

/** 기억한 호스트와 기기 증표를 버린다. 다른 데이터의 호스트에 새로 승인받을 때(BR-H5). */
export function forgetHost(): Promise<void> {
  return invokeCommand<void>("forget_host");
}
