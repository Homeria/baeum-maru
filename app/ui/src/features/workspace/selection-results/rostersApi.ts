import { postForFile } from "../../../api/client";
import type { RosterExportRequest } from "../../../api/generated/RosterExportRequest";

/**
 * 학기의 결과 명단을 Excel 파일 내용으로 받는다(BR-P1).
 *
 * 원본 명단은 관리자·직원만 받는다(BR-P10). 받을 때마다 호스트가 내보내기 기록을 남긴다(BR-P11).
 * 선발을 마감한 강좌가 없으면 `NO_CLOSED_COURSES` 오류.
 */
export function exportRoster(
  termId: number,
  request: RosterExportRequest,
): Promise<Uint8Array> {
  return postForFile(`/api/terms/${termId}/roster-exports`, request);
}
