import { getJson, postJson } from "./client";
import type { SetupRequest } from "./generated/SetupRequest";
import type { SetupResponse } from "./generated/SetupResponse";
import type { SetupStatusResponse } from "./generated/SetupStatusResponse";

/** 처음 설정이 필요한지 묻는다. 직원이 하나도 없으면 필요하다. */
export function fetchSetupStatus(): Promise<SetupStatusResponse> {
  return getJson<SetupStatusResponse>("/api/setup");
}

/** 첫 관리자를 만들고 복구 코드를 받는다 (BR-K11). 복구 코드는 이때 한 번만 온다. */
export function setup(request: SetupRequest): Promise<SetupResponse> {
  return postJson<SetupResponse>("/api/setup", request);
}
