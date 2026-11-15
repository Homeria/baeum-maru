import { describe, expect, test } from "vitest";
import { searchQueryFrom } from "./useSearchText";

describe("searchQueryFrom", () => {
  test("앞뒤 공백을 뺀다", () => {
    expect(searchQueryFrom("  회원  ")).toBe("회원");
  });

  test("끝에 남은 한글 낱자를 뺀다", () => {
    expect(searchQueryFrom("회원ㅇ")).toBe("회원");
    expect(searchQueryFrom("회원ㅇㅏ")).toBe("회원");
  });

  test("완성된 글자는 조합 중이어도 그대로 쓴다", () => {
    expect(searchQueryFrom("회원")).toBe("회원");
  });

  test("낱자만 있으면 빈 검색어", () => {
    expect(searchQueryFrom("ㅎ")).toBe("");
  });

  test("숫자와 하이픈은 그대로", () => {
    expect(searchQueryFrom("010-0000")).toBe("010-0000");
  });
});
