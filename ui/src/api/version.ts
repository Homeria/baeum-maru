import { getJson } from "./client";
import type { VersionResponse } from "./generated/VersionResponse";

/** 호스트 버전과 호환 여부를 받는다. */
export function fetchHostVersion(): Promise<VersionResponse> {
  return getJson<VersionResponse>("/api/version");
}
