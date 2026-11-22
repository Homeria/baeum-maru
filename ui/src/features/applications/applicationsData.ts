import type { ApplicationCountResponse } from "../../api/generated/ApplicationCountResponse";
import type { ApplicationRuleCode } from "../../api/generated/ApplicationRuleCode";

/** 신청 규칙의 화면 이름. 용어집의 "신청 규칙 종류"를 따른다. */
export const RULE_LABELS: Record<ApplicationRuleCode, string> = {
  no_duplicate: "같은 강좌 두 번",
  no_time_overlap: "시간 겹침",
  no_same_subject: "같은 과목 중복",
};

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
