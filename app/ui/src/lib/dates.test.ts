import { describe, expect, test } from "vitest";
import { todayText } from "./dates";

describe("todayText", () => {
  test("이 PC의 날짜를 YYYY-MM-DD로", () => {
    expect(todayText(new Date(2027, 0, 5, 23, 59))).toBe("2027-01-05");
  });
});
