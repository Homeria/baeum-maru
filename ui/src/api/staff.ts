import { getJson, patchJson, postJson, postNoContent } from "./client";
import type { CreateStaffRequest } from "./generated/CreateStaffRequest";
import type { CreateStaffResponse } from "./generated/CreateStaffResponse";
import type { StaffListResponse } from "./generated/StaffListResponse";
import type { StaffResponse } from "./generated/StaffResponse";
import type { UpdateStaffRequest } from "./generated/UpdateStaffRequest";

/** 직원을 모두 받는다. 사용 중인 직원 먼저, 이름 순서. 관리자만. */
export function listStaff(): Promise<StaffListResponse> {
  return getJson<StaffListResponse>("/api/staff");
}

/** 새 직원을 저장하고 새 id를 받는다. */
export function createStaff(
  request: CreateStaffRequest,
): Promise<CreateStaffResponse> {
  return postJson<CreateStaffResponse>("/api/staff", request);
}

/** 직원의 이름·역할·유효기간·사용 중지를 바꾸고 바꾼 뒤 모습을 받는다. */
export function updateStaff(
  id: number,
  request: UpdateStaffRequest,
): Promise<StaffResponse> {
  return patchJson<StaffResponse>(`/api/staff/${id}`, request);
}

/** BR-K10: 직원의 PIN을 새로 정한다. 계정 잠금도 풀린다. */
export function resetStaffPin(id: number, pin: string): Promise<void> {
  return postNoContent(`/api/staff/${id}/pin`, { pin });
}
