import { describe, expect, test } from "vitest";
import { membersSearchPath } from "./members";

describe("membersSearchPath", () => {
  test("조건이 없으면 주소만", () => {
    expect(membersSearchPath({})).toBe("/api/members");
  });

  test("빈 검색어와 공백뿐인 검색어는 싣지 않는다", () => {
    expect(membersSearchPath({ query: "" })).toBe("/api/members");
    expect(membersSearchPath({ query: "  " })).toBe("/api/members");
  });

  test("검색어 앞뒤 공백을 지우고 한글은 인코딩한다", () => {
    expect(membersSearchPath({ query: " 회원 " })).toBe(
      "/api/members?query=%ED%9A%8C%EC%9B%90",
    );
  });

  test("쪽 번호와 한 쪽 인원은 camelCase로 싣는다", () => {
    expect(membersSearchPath({ query: "010", page: 2, pageSize: 20 })).toBe(
      "/api/members?query=010&page=2&pageSize=20",
    );
  });
});
