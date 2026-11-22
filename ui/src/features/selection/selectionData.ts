import type { CourseLotteryResponse } from "../../api/generated/CourseLotteryResponse";
import type { CourseStageResponse } from "../../api/generated/CourseStageResponse";
import type { EntrantResponse } from "../../api/generated/EntrantResponse";
import type { SelectionStageCode } from "../../api/generated/SelectionStageCode";

/**
 * 추첨 화면에서 신청자 목록을 섞어 보여 주는 간격(밀리초).
 *
 * 보여 주기만 할 뿐 결과와 상관없다(ADR 0016). 설정 기능이 생기면 옮긴다.
 */
export const SHUFFLE_INTERVAL_MS = 250;

/** 선발 단계의 화면 이름. */
export const STAGE_LABELS: Record<SelectionStageCode, string> = {
  not_drawn: "추첨 전",
  drawn: "추첨함",
  closed: "마감",
};

/**
 * 추첨 화면의 한 줄. 예: `회원1(12-12345)`
 *
 * 이름 옆에 붙일 정보는 지금 회원번호로 정해 두었다. 설정 기능이 생기면 고를 수 있게 한다.
 */
export function entrantLabel(
  entrant: Pick<EntrantResponse, "memberName" | "memberNo">,
): string {
  return `${entrant.memberName}(${entrant.memberNo})`;
}

/** 강좌 id로 선발 단계를 찾는 표. 목록에 없는 강좌는 찾지 못한다. */
export function stagesByCourse(
  stages: readonly CourseStageResponse[],
): ReadonlyMap<number, SelectionStageCode> {
  return new Map(stages.map((stage) => [stage.courseId, stage.stage]));
}

/** 뽑힌 순서를 당첨과 대기로 나눈다. 대기는 앞에서부터 대기 순번 1, 2, … */
export function splitResult(
  entrants: readonly EntrantResponse[],
  selectedCount: number,
): {
  selected: readonly EntrantResponse[];
  waitlisted: readonly EntrantResponse[];
} {
  return {
    selected: entrants.slice(0, selectedCount),
    waitlisted: entrants.slice(selectedCount),
  };
}

/**
 * `items`를 섞은 새 목록. 화면에서 섞이는 모습을 보여 줄 때만 쓴다.
 *
 * 실제 결과는 호스트가 멈춤을 누른 순간 정한다(ADR 0016).
 */
export function shuffled<T>(
  items: readonly T[],
  random: () => number = Math.random,
): T[] {
  const result = [...items];
  for (let index = result.length - 1; index > 0; index -= 1) {
    const other = Math.floor(random() * (index + 1));
    [result[index], result[other]] = [result[other], result[index]];
  }
  return result;
}

/**
 * 추첨 화면 아래에 둘 버튼.
 *
 * - `start`: 시작 → 섞기 → 멈춤
 * - `selectAll`: 신청자가 정원 이하라 섞지 않고 모두 당첨(BR-S6)
 * - `redraw`: 추첨한 뒤. 다시 추첨(BR-L4)과 선발 마감(BR-L5)
 * - `none`: 마감했거나, 신청이 없거나, 이 PC·역할로는 할 수 없음(BR-L6)
 */
export type LotteryButtons = "start" | "selectAll" | "redraw" | "none";

/** 추첨 화면의 상태에 맞는 버튼. */
export function lotteryButtons(lottery: CourseLotteryResponse): LotteryButtons {
  if (
    !lottery.canDraw ||
    lottery.stage === "closed" ||
    lottery.entrants.length === 0
  ) {
    return "none";
  }
  if (lottery.stage === "drawn") {
    return "redraw";
  }
  return lottery.entrants.length <= lottery.capacity ? "selectAll" : "start";
}

/** 섞는 모습 없이 바로 뽑아도 되는지. 신청자가 정원 이하이면 모두 당첨이라 섞을 까닭이 없다. */
export function needsShuffle(lottery: CourseLotteryResponse): boolean {
  return lottery.entrants.length > lottery.capacity;
}
