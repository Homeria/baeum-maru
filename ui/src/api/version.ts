import { getJson } from "./client";

/** `/api/version` 응답. */
export interface VersionResponse {
  hostVersion: string;
  /** 이 앱과 호스트를 함께 쓸 수 있는지 */
  compatible: boolean;
}

/** 호스트 버전과 호환 여부를 받는다. */
export function fetchHostVersion(): Promise<VersionResponse> {
  return getJson<VersionResponse>("/api/version");
}
