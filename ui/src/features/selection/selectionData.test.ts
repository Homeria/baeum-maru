import { describe, expect, test } from "vitest";
import type { CourseLotteryResponse } from "../../api/generated/CourseLotteryResponse";
import type { EntrantResponse } from "../../api/generated/EntrantResponse";
import {
  entrantLabel,
  lotteryButtons,
  needsShuffle,
  resultColumns,
  shuffled,
  splitResult,
  stagesByCourse,
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

describe("stagesByCourse", () => {
  test("강좌 id로 단계를 찾는다", () => {
    const stages = stagesByCourse([
      { courseId: 1, stage: "drawn" },
      { courseId: 2, stage: "closed" },
    ]);

    expect(stages.get(1)).toBe("drawn");
    expect(stages.get(2)).toBe("closed");
    expect(stages.get(3)).toBeUndefined();
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
