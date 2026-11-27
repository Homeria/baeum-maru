import { describe, expect, test } from "vitest";
import { FLOORS, floorLabel } from "./floor";

describe("floorLabel", () => {
  test("음수는 지하로 보여 준다", () => {
    expect(floorLabel(-1)).toBe("지하 1층");
    expect(floorLabel(-2)).toBe("지하 2층");
  });

  test("양수는 층 번호 그대로", () => {
    expect(floorLabel(3)).toBe("3층");
  });

  test("층이 없으면 빈 글자", () => {
    expect(floorLabel(null)).toBe("");
  });
});

test("층 선택지에 0층은 없다", () => {
  expect(FLOORS).not.toContain(0);
  expect(FLOORS.map(floorLabel)).toEqual([
    "지하 2층",
    "지하 1층",
    "1층",
    "2층",
    "3층",
    "4층",
    "5층",
  ]);
});
