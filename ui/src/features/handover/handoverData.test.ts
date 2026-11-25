import { describe, expect, test } from "vitest";
import { fileLines } from "./handoverData";

describe("fileLines", () => {
  test("학기와 회원·신청 수를 보여 준다", () => {
    expect(
      fileLines({
        termNames: ["2026년 1학기", "2026년 2학기"],
        memberCount: 120,
        applicationCount: 340,
      }),
    ).toEqual(["학기: 2026년 1학기, 2026년 2학기", "회원 120명, 신청 340건"]);
  });

  test("학기가 없으면 없음", () => {
    expect(
      fileLines({ termNames: [], memberCount: 0, applicationCount: 0 })[0],
    ).toBe("학기: 없음");
  });
});
