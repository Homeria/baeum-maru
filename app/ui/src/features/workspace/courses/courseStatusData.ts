import type { ApplicationCountResponse } from "../../../api/generated/ApplicationCountResponse";
import type { CourseStageResponse } from "../../../api/generated/CourseStageResponse";
import type { SelectionStageCode } from "../../../api/generated/SelectionStageCode";

/** 선발 단계의 화면 이름. */
export const STAGE_LABELS: Record<SelectionStageCode, string> = {
  not_drawn: "추첨 전",
  drawn: "추첨함",
  closed: "마감",
};

/** 강좌 id로 선발 단계를 찾는 표. 목록에 없는 강좌는 찾지 못한다. */
export function stagesByCourse(
  stages: readonly CourseStageResponse[],
): ReadonlyMap<number, SelectionStageCode> {
  return new Map(stages.map((stage) => [stage.courseId, stage.stage]));
}

/** 강좌 id로 신청 인원을 찾는 표. 목록에 없는 강좌는 0명이다. */
export function countsByCourse(
  counts: readonly ApplicationCountResponse[],
): ReadonlyMap<number, number> {
  return new Map(counts.map((count) => [count.courseId, count.count]));
}

/** 신청 인원과 정원. 예: `12/20` */
export function countLabel(count: number, capacity: number): string {
  return `${count}/${capacity}`;
}
