import { describe, expect, test } from "vitest";
import { countLabel, countsByCourse } from "./applicationsData";

describe("countsByCourse", () => {
  test("강좌 id로 인원을 찾고, 없는 강좌는 찾지 못한다", () => {
    const counts = countsByCourse([
      { courseId: 1, count: 12 },
      { courseId: 3, count: 1 },
    ]);

    expect(counts.get(1)).toBe(12);
    expect(counts.get(3)).toBe(1);
    expect(counts.get(2)).toBeUndefined();
  });
});

describe("countLabel", () => {
  test("신청 인원/정원으로 보여 준다", () => {
    expect(countLabel(12, 20)).toBe("12/20");
  });
});
