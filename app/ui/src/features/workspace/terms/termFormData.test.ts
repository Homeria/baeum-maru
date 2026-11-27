import { describe, expect, test } from "vitest";
import { HostError } from "../../../api/client";
import type { TermResponse } from "../../../api/generated/TermResponse";
import {
  currentTerm,
  termFormErrorsFrom,
  termPeriodLabel,
  toTermRequest,
} from "./termFormData";

function term(id: number, name: string, startsOn: string): TermResponse {
  return { id, name, startsOn, endsOn: "2027-12-31" };
}

const TERMS = [
  term(2, "2027년 2학기", "2027-09-01"),
  term(1, "2027년 1학기", "2027-03-02"),
];

describe("currentTerm", () => {
  test("고른 학기가 있으면 그 학기", () => {
    expect(currentTerm(TERMS, 1)?.name).toBe("2027년 1학기");
  });

  test("고르지 않았으면 시작일이 가장 늦은 학기(목록 맨 앞)", () => {
    expect(currentTerm(TERMS, null)?.name).toBe("2027년 2학기");
  });

  test("고른 학기가 지워졌으면 맨 앞 학기로", () => {
    expect(currentTerm(TERMS, 99)?.name).toBe("2027년 2학기");
  });

  test("학기가 없으면 null", () => {
    expect(currentTerm([], null)).toBeNull();
  });
});

describe("toTermRequest", () => {
  test("비운 날짜는 null", () => {
    expect(
      toTermRequest({ name: "2027년 1학기", startsOn: "", endsOn: "" }),
    ).toEqual({ name: "2027년 1학기", startsOn: null, endsOn: null });
  });
});

describe("termPeriodLabel", () => {
  test("시작일 ~ 종료일", () => {
    expect(termPeriodLabel(term(1, "2027년 1학기", "2027-09-01"))).toBe(
      "2027-09-01 ~ 2027-12-31",
    );
  });
});

describe("termFormErrorsFrom", () => {
  test("이름이 겹치면 이름 칸에 붙인다 (BR-T1)", () => {
    const error = new HostError(
      409,
      "같은 이름의 학기가 있습니다",
      "DUPLICATE_TERM_NAME",
      "BR-T1",
    );

    expect(termFormErrorsFrom(error).fields).toEqual({
      name: "같은 이름의 학기가 있습니다",
    });
  });

  test("강좌가 있는 학기를 지우려 하면 전체 문구로 (BR-T5)", () => {
    const message =
      "강좌가 있는 학기는 지울 수 없습니다. 강좌를 먼저 지워 주세요";
    const error = new HostError(409, message, "TERM_IN_USE", "BR-T5");

    expect(termFormErrorsFrom(error)).toEqual({ fields: {}, form: message });
  });
});
