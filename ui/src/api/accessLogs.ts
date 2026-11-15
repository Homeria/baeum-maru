import { getJson } from "./client";
import type { AccessLogListResponse } from "./generated/AccessLogListResponse";

/** 접속 기록을 최근 것부터 `page`쪽(1부터) 받는다. 관리자만. */
export function listAccessLogs(
  page: number,
  pageSize: number,
): Promise<AccessLogListResponse> {
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
  });
  return getJson<AccessLogListResponse>(`/api/access-logs?${params}`);
}
