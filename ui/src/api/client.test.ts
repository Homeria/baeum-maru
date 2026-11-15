import { describe, expect, test } from "vitest";
import { hostErrorFrom } from "./client";

describe("hostErrorFrom", () => {
  test("호스트 오류 모양이면 내용을 그대로 담는다", () => {
    const error = hostErrorFrom(422, {
      code: "INVALID_INPUT",
      message: "입력을 확인해 주세요",
      rule: null,
      fields: [{ field: "name", message: "이름을 입력해 주세요" }],
    });

    expect(error.status).toBe(422);
    expect(error.code).toBe("INVALID_INPUT");
    expect(error.message).toBe("입력을 확인해 주세요");
    expect(error.fields).toEqual([
      { field: "name", message: "이름을 입력해 주세요" },
    ]);
  });

  test("업무 규칙 번호를 담는다", () => {
    const error = hostErrorFrom(409, {
      code: "DUPLICATE_MEMBER_NO",
      message: "이미 있는 회원번호입니다",
      rule: "BR-M2",
      fields: [],
    });

    expect(error.rule).toBe("BR-M2");
  });

  test("본문이 없으면 일반 문구를 쓴다", () => {
    const error = hostErrorFrom(502, null);

    expect(error.code).toBeNull();
    expect(error.message).toContain("502");
    expect(error.fields).toEqual([]);
  });

  test("모양이 다른 본문은 호스트 오류로 보지 않는다", () => {
    const error = hostErrorFrom(500, { message: "다른 서버의 오류" });

    expect(error.code).toBeNull();
    expect(error.message).not.toBe("다른 서버의 오류");
  });
});
