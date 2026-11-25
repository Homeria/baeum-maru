import { describe, expect, test } from "vitest";
import type { CourseLotteryResponse } from "../../api/generated/CourseLotteryResponse";
import type { EntrantResponse } from "../../api/generated/EntrantResponse";
import {
  drawPool,
  entrantLabel,
  guaranteedOf,
  remainingSeats,
  lotteryButtons,
  needsShuffle,
  resultColumns,
  shuffled,
  splitResult,
} from "./selectionData";

/** 지어낸 신청자 `count`명. */
function entrants(count: number): EntrantResponse[] {
  return Array.from({ length: count }, (_, index) => ({
    applicationId: index + 1,
    memberName: `회원${index + 1}`,
    memberNo: `12-${String(index + 1).padStart(5, "0")}`,
    guaranteed: false,
  }));
}

/** 앞에서 `guaranteed`명이 자리 보장인 신청자 `count`명. */
function withGuaranteed(count: number, guaranteed: number): EntrantResponse[] {
  return entrants(count).map((entrant, index) => ({
    ...entrant,
    guaranteed: index < guaranteed,
  }));
}

function lottery(
  overrides: Partial<CourseLotteryResponse>,
): CourseLotteryResponse {
  return {
    capacity: 2,
    stage: "not_drawn",
    entrants: entrants(3),
    selectedCount: 0,
    canDraw: true,
    ...overrides,
  };
}

describe("entrantLabel", () => {
  test("이름(회원번호)", () => {
    expect(entrantLabel({ memberName: "회원1", memberNo: "12-12345" })).toBe(
      "회원1(12-12345)",
    );
  });
});

describe("splitResult", () => {
  test("앞에서 당첨 인원만큼 당첨, 나머지는 순서대로 대기", () => {
    const { selected, waitlisted } = splitResult(entrants(5), 2);

    expect(selected.map((e) => e.applicationId)).toEqual([1, 2]);
    expect(waitlisted.map((e) => e.applicationId)).toEqual([3, 4, 5]);
  });

  test("추첨 전(당첨 0명)이면 당첨이 없다", () => {
    expect(splitResult(entrants(2), 0).selected).toEqual([]);
  });
});

describe("resultColumns", () => {
  const ids = (list: readonly EntrantResponse[]) =>
    list.map((entrant) => entrant.applicationId);

  test("추첨 전 정원보다 많으면 둘 다 비운다", () => {
    const result = resultColumns(lottery({}), false);

    expect(result).toEqual({ selected: [], waitlisted: [] });
  });

  test("추첨 전 정원 이하이면 모두 당첨자 칸에 미리 보인다", () => {
    const result = resultColumns(lottery({ entrants: entrants(2) }), false);

    expect(ids(result.selected)).toEqual([1, 2]);
    expect(result.waitlisted).toEqual([]);
  });

  test("섞는 중에는 둘 다 비운다", () => {
    const drawn = lottery({ stage: "drawn", selectedCount: 2 });

    expect(resultColumns(drawn, true)).toEqual({
      selected: [],
      waitlisted: [],
    });
  });

  test("추첨 뒤에는 뽑힌 순서를 당첨과 대기로 나눈다", () => {
    const result = resultColumns(
      lottery({ stage: "drawn", selectedCount: 2 }),
      false,
    );

    expect(ids(result.selected)).toEqual([1, 2]);
    expect(ids(result.waitlisted)).toEqual([3]);
  });
});

describe("shuffled", () => {
  test("모두 그대로 있고 원래 목록은 바꾸지 않는다", () => {
    const original = [1, 2, 3, 4, 5];

    const result = shuffled(original);

    expect([...result].sort()).toEqual(original);
    expect(original).toEqual([1, 2, 3, 4, 5]);
  });

  test("난수가 늘 0이면 맨 앞이 차례로 끝과 바뀐다", () => {
    expect(shuffled([1, 2, 3], () => 0)).toEqual([2, 3, 1]);
  });
});

describe("lotteryButtons", () => {
  test("정원보다 많으면 시작, 정원 이하이면 전원 당첨", () => {
    expect(lotteryButtons(lottery({}))).toBe("start");
    expect(lotteryButtons(lottery({ entrants: entrants(2) }))).toBe(
      "selectAll",
    );
  });

  test("추첨한 뒤에는 다시 추첨·마감", () => {
    expect(lotteryButtons(lottery({ stage: "drawn", selectedCount: 2 }))).toBe(
      "redraw",
    );
  });

  test("마감, 신청 없음, 이 PC·역할로 할 수 없음은 버튼 없음", () => {
    expect(lotteryButtons(lottery({ stage: "closed" }))).toBe("none");
    expect(lotteryButtons(lottery({ entrants: [] }))).toBe("none");
    expect(lotteryButtons(lottery({ canDraw: false }))).toBe("none");
  });
});

describe("needsShuffle", () => {
  test("정원보다 많을 때만 섞는다", () => {
    expect(needsShuffle(lottery({}))).toBe(true);
    expect(needsShuffle(lottery({ entrants: entrants(2) }))).toBe(false);
  });
});

describe("자리 보장(BR-S4, BR-S5)", () => {
  test("자리 보장과 추첨할 신청을 나누고 남은 자리를 센다", () => {
    const current = lottery({ capacity: 3, entrants: withGuaranteed(5, 1) });

    expect(guaranteedOf(current.entrants).map((e) => e.applicationId)).toEqual([
      1,
    ]);
    expect(drawPool(current.entrants)).toHaveLength(4);
    expect(remainingSeats(current)).toBe(2);
  });

  test("추첨 전과 섞는 중에도 자리 보장은 당첨자 칸 맨 위에 있다", () => {
    const current = lottery({ capacity: 3, entrants: withGuaranteed(5, 1) });

    for (const shuffling of [false, true]) {
      const { selected, waitlisted } = resultColumns(current, shuffling);
      expect(selected.map((e) => e.applicationId)).toEqual([1]);
      expect(waitlisted).toEqual([]);
    }
  });

  test("추첨할 신청이 남은 자리 이하이면 섞지 않고 전원 당첨", () => {
    const current = lottery({ capacity: 3, entrants: withGuaranteed(3, 1) });

    expect(needsShuffle(current)).toBe(false);
    expect(lotteryButtons(current)).toBe("selectAll");
    expect(resultColumns(current, false).selected).toHaveLength(3);
  });

  test("남은 자리보다 많으면 섞는다", () => {
    const current = lottery({ capacity: 3, entrants: withGuaranteed(5, 2) });

    expect(needsShuffle(current)).toBe(true);
    expect(lotteryButtons(current)).toBe("start");
  });

  test("자리 보장이 정원보다 많으면 추첨할 수 없고 모두에게 알린다", () => {
    const current = lottery({ capacity: 2, entrants: withGuaranteed(4, 3) });

    expect(remainingSeats(current)).toBe(0);
    expect(lotteryButtons(current)).toBe("overCapacity");
    expect(lotteryButtons({ ...current, canDraw: false })).toBe("overCapacity");
  });
});
