import { invoke } from "@tauri-apps/api/core";
import { getJson, postNoContent, sendDelete } from "../../../api/client";
import type { DeviceListResponse } from "../../../api/generated/DeviceListResponse";
import type { DeviceRequestListResponse } from "../../../api/generated/DeviceRequestListResponse";

/** BR-K15: 기다리는 승인 요청. 관리자가 호스트 PC에서만 본다. */
export function listDeviceRequests(): Promise<DeviceRequestListResponse> {
  return getJson<DeviceRequestListResponse>("/api/device-requests");
}

/** BR-K15: 관리자의 PIN을 넣어 승인 요청을 허락한다. */
export function approveDevice(id: number, pin: string): Promise<void> {
  return postNoContent(`/api/device-requests/${id}/approve`, { pin });
}

/** BR-K15: 승인 요청을 거절한다. */
export function rejectDevice(id: number): Promise<void> {
  return postNoContent(`/api/device-requests/${id}/reject`, {});
}

/** BR-K16: 승인한 기기. 허락한 순서. */
export function listDevices(): Promise<DeviceListResponse> {
  return getJson<DeviceListResponse>("/api/devices");
}

/** BR-K16: 기기 승인을 취소한다. 그 기기는 다음 요청부터 막힌다. */
export function revokeDevice(id: number): Promise<void> {
  return sendDelete(`/api/devices/${id}`);
}

/**
 * 다른 PC가 입력할 이 PC의 주소(예: `192.168.0.12`). 찾지 못하면 `null`.
 *
 * 호스트 PC의 앱에 묻는다(ADR 0004).
 */
export async function hostAddress(): Promise<string | null> {
  return invoke<string | null>("host_address");
}

/**
 * 기다리는 승인 요청 수. 호스트 PC의 앱에 묻는다.
 *
 * 서버가 아니라 앱에 물어서, 자주 물어도 로그인 세션의 마지막 요청 시각이 바뀌지 않는다(BR-K8).
 */
export function openDeviceRequestCount(): Promise<number> {
  return invoke<number>("open_device_request_count");
}
