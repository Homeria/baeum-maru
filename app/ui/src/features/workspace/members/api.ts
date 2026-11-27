import { getJson, patchJson, postJson, sendDelete } from "../../../api/client";
import type { CreateMemberResponse } from "../../../api/generated/CreateMemberResponse";
import type { MemberListResponse } from "../../../api/generated/MemberListResponse";
import type { MemberRequest } from "../../../api/generated/MemberRequest";
import type { MemberResponse } from "../../../api/generated/MemberResponse";

/** 회원 찾기 조건. 빠진 값은 호스트의 기본값을 쓴다. */
export interface MemberSearch {
  query?: string;
  page?: number;
  pageSize?: number;
}

/** 회원 찾기 요청 주소. 빈 검색어는 싣지 않는다. */
export function membersSearchPath({
  query,
  page,
  pageSize,
}: MemberSearch): string {
  const params = new URLSearchParams();
  const trimmed = query?.trim();
  if (trimmed) {
    params.set("query", trimmed);
  }
  if (page !== undefined) {
    params.set("page", String(page));
  }
  if (pageSize !== undefined) {
    params.set("pageSize", String(pageSize));
  }
  const search = params.toString();
  return search ? `/api/members?${search}` : "/api/members";
}

/** 검색어로 회원을 찾는다. */
export function searchMembers(
  search: MemberSearch,
): Promise<MemberListResponse> {
  return getJson<MemberListResponse>(membersSearchPath(search));
}

/** 회원 한 명을 받는다. */
export function getMember(id: number): Promise<MemberResponse> {
  return getJson<MemberResponse>(`/api/members/${id}`);
}

/** 새 회원을 저장하고 새 id를 받는다. */
export function createMember(
  request: MemberRequest,
): Promise<CreateMemberResponse> {
  return postJson<CreateMemberResponse>("/api/members", request);
}

/** 회원의 칸을 모두 바꾸고 고친 뒤 모습을 받는다. */
export function updateMember(
  id: number,
  request: MemberRequest,
): Promise<MemberResponse> {
  return patchJson<MemberResponse>(`/api/members/${id}`, request);
}

/** 기록이 없는 회원을 지운다 (BR-M5). */
export function deleteMember(id: number): Promise<void> {
  return sendDelete(`/api/members/${id}`);
}
