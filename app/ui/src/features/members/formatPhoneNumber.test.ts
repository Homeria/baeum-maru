import { describe, expect, test } from "vitest";
import { formatPhoneNumber } from "./formatPhoneNumber";

describe("formatPhoneNumber", () => {
  test("휴대폰 11자리는 3-4-4", () => {
    expect(formatPhoneNumber("01000000001")).toBe("010-0000-0001");
  });

  test("지역번호 10자리는 3-3-4", () => {
    expect(formatPhoneNumber("0320000000")).toBe("032-000-0000");
  });

  test("서울 9자리는 2-3-4", () => {
    expect(formatPhoneNumber("020000000")).toBe("02-000-0000");
  });

  test("서울 10자리는 2-4-4", () => {
    expect(formatPhoneNumber("0200000000")).toBe("02-0000-0000");
  });

  test("모르는 모양은 그대로", () => {
    expect(formatPhoneNumber("031000000")).toBe("031000000");
    expect(formatPhoneNumber("010-0000-0001")).toBe("010-0000-0001");
    expect(formatPhoneNumber("")).toBe("");
  });
});
