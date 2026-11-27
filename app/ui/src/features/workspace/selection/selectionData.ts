import type { CourseLotteryResponse } from "../../../api/generated/CourseLotteryResponse";
import type { EntrantResponse } from "../../../api/generated/EntrantResponse";

/**
 * 추첨 화면에서 신청자 목록을 섞어 보여 주는 간격(밀리초).
 *
 * 보여 주기만 할 뿐 결과와 상관없다(ADR 0016). 설정 기능이 생기면 옮긴다.
 */
export const SHUFFLE_INTERVAL_MS = 250;

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

/** 자리 보장 신청(BR-S4). 호스트가 늘 맨 앞에 둔다. */
export function guaranteedOf(
  entrants: readonly EntrantResponse[],
): readonly EntrantResponse[] {
  return entrants.filter((entrant) => entrant.guaranteed);
}

/** 추첨할 신청. 자리 보장이 아닌 신청이다. */
export function drawPool(
  entrants: readonly EntrantResponse[],
): readonly EntrantResponse[] {
  return entrants.filter((entrant) => !entrant.guaranteed);
}

/** 자리 보장 신청이 정원보다 많아 추첨할 수 없는지(BR-S5). */
export function overCapacity(lottery: CourseLotteryResponse): boolean {
  return guaranteedOf(lottery.entrants).length > lottery.capacity;
}

/** 추첨으로 채울 자리. 정원에서 자리 보장 인원을 뺀다. 모자라면 0. */
export function remainingSeats(lottery: CourseLotteryResponse): number {
  return Math.max(0, lottery.capacity - guaranteedOf(lottery.entrants).length);
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
 * 당첨자·대기자 칸에 보여 줄 것.
 *
 * 자리 보장 신청(BR-S4)은 추첨과 상관없이 당첨이라, 추첨 전과 섞는 중에도 당첨자 칸 맨 위에 둔다.
 *
 * - 섞는 중: 자리 보장만 보이고 대기자는 비운다 (재추첨 포함).
 * - 추첨 전: 추첨할 신청이 남은 자리 이하이면 모두 당첨될 사람이라 당첨자 칸에 미리 보여
 *   준다(BR-S6). 기록은 "전원 당첨"을 눌러야 남는다. 남은 자리보다 많으면 자리 보장만 보인다.
 * - 추첨 뒤: 뽑힌 순서를 당첨과 대기로 나눈다. 호스트가 자리 보장을 맨 앞에 둔다.
 */
export function resultColumns(
  lottery: CourseLotteryResponse,
  shuffling: boolean,
): {
  selected: readonly EntrantResponse[];
  waitlisted: readonly EntrantResponse[];
} {
  if (shuffling) {
    return { selected: guaranteedOf(lottery.entrants), waitlisted: [] };
  }
  if (lottery.stage === "not_drawn") {
    return needsShuffle(lottery) || overCapacity(lottery)
      ? { selected: guaranteedOf(lottery.entrants), waitlisted: [] }
      : { selected: lottery.entrants, waitlisted: [] };
  }
  return splitResult(lottery.entrants, lottery.selectedCount);
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
 * - `selectAll`: 추첨할 신청이 남은 자리 이하라 섞지 않고 모두 당첨(BR-S6)
 * - `redraw`: 추첨한 뒤. 다시 추첨(BR-L4)과 선발 마감(BR-L5)
 * - `overCapacity`: 추첨 전인데 자리 보장 신청이 정원보다 많아 추첨할 수 없음(BR-S5).
 *   이 PC·역할과 상관없이 모두에게 까닭을 보여 준다
 * - `none`: 마감했거나, 신청이 없거나, 이 PC·역할로는 할 수 없음(BR-L6)
 */
export type LotteryButtons =
  "start" | "selectAll" | "redraw" | "overCapacity" | "none";

/** 추첨 화면의 상태에 맞는 버튼. */
export function lotteryButtons(lottery: CourseLotteryResponse): LotteryButtons {
  if (lottery.stage === "closed" || lottery.entrants.length === 0) {
    return "none";
  }
  if (lottery.stage === "not_drawn" && overCapacity(lottery)) {
    return "overCapacity";
  }
  if (!lottery.canDraw) {
    return "none";
  }
  if (lottery.stage === "drawn") {
    return "redraw";
  }
  return needsShuffle(lottery) ? "start" : "selectAll";
}

/**
 * 섞는 모습을 보여 줄지. 추첨할 신청이 남은 자리 이하이면 모두 당첨이라 섞을 까닭이 없다.
 */
export function needsShuffle(lottery: CourseLotteryResponse): boolean {
  return drawPool(lottery.entrants).length > remainingSeats(lottery);
}
