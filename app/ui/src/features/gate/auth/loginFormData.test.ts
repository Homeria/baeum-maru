import { describe, expect, test } from "vitest";
import { HostError } from "../../../api/client";
import {
  groupByRole,
  loginFormErrorsFrom,
  toLoginRequest,
} from "./loginFormData";

describe("toLoginRequest", () => {
  test("직원을 고르지 않았으면 이름 칸 오류", () => {
    expect(toLoginRequest(null, "1234")).toEqual({
      fields: { staffId: "이름을 골라 주세요" },
      form: null,
    });
  });

  test("고른 직원과 PIN을 담는다", () => {
    expect(toLoginRequest(3, "1234")).toEqual({ staffId: 3, pin: "1234" });
  });
});

describe("loginFormErrorsFrom", () => {
  test("PIN이 틀리면 PIN 칸에 붙인다", () => {
    const error = new HostError(401, "PIN이 맞지 않습니다", "WRONG_PIN");

    expect(loginFormErrorsFrom(error).fields).toEqual({
      pin: "PIN이 맞지 않습니다",
    });
  });

  test("계정이 잠겼으면 전체 문구로 보여 준다", () => {
    const error = new HostError(
      403,
      "PIN을 5번 연속 틀려 잠겼습니다",
      "ACCOUNT_LOCKED",
      "BR-K7",
    );

    expect(loginFormErrorsFrom(error)).toEqual({
      fields: {},
      form: "PIN을 5번 연속 틀려 잠겼습니다",
    });
  });
});

describe("groupByRole", () => {
  test("관리자부터 역할별로 묶고 묶음 안은 받은 순서를 지킨다", () => {
    const groups = groupByRole([
      { id: 1, name: "가임시", role: "temp_staff" },
      { id: 2, name: "나관리", role: "admin" },
      { id: 3, name: "다임시", role: "temp_staff" },
    ]);

    expect(groups).toEqual([
      { role: "admin", items: [{ id: 2, name: "나관리", role: "admin" }] },
      {
        role: "temp_staff",
        items: [
          { id: 1, name: "가임시", role: "temp_staff" },
          { id: 3, name: "다임시", role: "temp_staff" },
        ],
      },
    ]);
  });

  test("직원이 없으면 묶음도 없다", () => {
    expect(groupByRole([])).toEqual([]);
  });
});
