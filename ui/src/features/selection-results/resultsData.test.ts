import { describe, expect, test } from "vitest";
import {
  matchesQuery,
  memberLabel,
  openSeatsLabel,
  restoreLabel,
} from "./resultsData";

const member = {
  applicationId: 1,
  memberName: "김 회원",
  memberNo: "12-00034",
};

describe("matchesQuery", () => {
  test("이름 일부나 회원번호 일부와 맞는다", () => {
    expect(matchesQuery(member, "회원")).toBe(true);
    expect(matchesQuery(member, "0034")).toBe(true);
    expect(matchesQuery(member, "박")).toBe(false);
  });

  test("띄어쓰기는 무시한다", () => {
    expect(matchesQuery(member, " 김회 ")).toBe(true);
  });

  test("빈 글자는 아무도 맞지 않는다", () => {
    expect(matchesQuery(member, "  ")).toBe(false);
  });
});

describe("글자", () => {
  test("회원은 이름(회원번호)", () => {
    expect(memberLabel(member)).toBe("김 회원(12-00034)");
  });

  test("빈자리가 없으면 없다고 쓴다", () => {
    expect(openSeatsLabel(0)).toBe("빈자리 없음");
    expect(openSeatsLabel(2)).toBe("빈자리 2");
  });
});

describe("restoreLabel", () => {
  test("당첨에서 포기했으면 당첨으로", () => {
    expect(
      restoreLabel({ member, withdrawnFrom: "selected", waitlistNumber: null }),
    ).toBe("당첨");
  });

  test("대기에서 포기했으면 원래 순번으로", () => {
    expect(
      restoreLabel({ member, withdrawnFrom: "waitlisted", waitlistNumber: 2 }),
    ).toBe("대기 2번");
  });
});
