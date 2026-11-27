import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";

/** 3단계 기초 정보 미리 넣기의 하위 단계. */
export type BasicsStep = "rooms" | "subjects";

/**
 * 첫 실행 3단계에서 추천 체크박스로 보여 줄 우선 배정 사유 이름.
 *
 * 체크하면 `자리 보장`으로 즉시 등록하고, 선택하지 않으면 저장하지 않는다.
 */
export const PRESET_PRIORITY_REASONS: readonly string[] = [
  "직전 학기 반장",
  "보훈",
  "장애",
  "기초생활수급자",
];

/** 추천 사유 중 이미 등록된 항목을 이름으로 찾는다. */
export function findPresetReason(
  reasons: readonly PriorityReasonResponse[],
  name: string,
): PriorityReasonResponse | undefined {
  return reasons.find((reason) => reason.name === name);
}

/** 추천 4개 외에 직원이 직접 추가한 우선 배정 사유 목록. */
export function customPriorityReasons(
  reasons: readonly PriorityReasonResponse[],
): readonly PriorityReasonResponse[] {
  return reasons.filter(
    (reason) => !PRESET_PRIORITY_REASONS.includes(reason.name),
  );
}
