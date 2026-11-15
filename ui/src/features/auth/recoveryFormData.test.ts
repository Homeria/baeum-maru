import { describe, expect, test } from "vitest";
import { HostError } from "../../api/client";
import {
  EMPTY_RECOVERY_FORM,
  recoveryFormErrorsFrom,
  toRecoveryRequest,
} from "./recoveryFormData";

const FILLED = {
  staffId: 1,
  code: "AAAA-BBBB-CCCC-DDDD",
  pin: "123456",
  pinConfirm: "123456",
};

describe("toRecoveryRequest", () => {
  test("관리자를 고르지 않았으면 이름 칸 오류", () => {
    expect(toRecoveryRequest(EMPTY_RECOVERY_FORM)).toEqual({
      fields: { staffId: "관리자 이름을 골라 주세요" },
      form: null,
    });
  });

  test("두 PIN이 다르면 PIN 확인 칸 오류", () => {
    const result = toRecoveryRequest({ ...FILLED, pinConfirm: "654321" });

    expect(result).toHaveProperty("fields.pinConfirm");
  });

  test("PIN 확인 칸은 보내지 않는다", () => {
    expect(toRecoveryRequest(FILLED)).toEqual({
      staffId: 1,
      code: "AAAA-BBBB-CCCC-DDDD",
      pin: "123456",
    });
  });
});

describe("recoveryFormErrorsFrom", () => {
  test("틀린 복구 코드는 코드 칸에 붙인다", () => {
    const error = new HostError(
      401,
      "복구 코드가 맞지 않습니다",
      "WRONG_RECOVERY_CODE",
      "BR-K12",
    );

    expect(recoveryFormErrorsFrom(error).fields).toEqual({
      code: "복구 코드가 맞지 않습니다",
    });
  });

  test("관리자가 아닌 직원을 골랐으면 이름 칸에 붙인다", () => {
    const error = new HostError(
      403,
      "관리자 PIN만 다시 정할 수 있습니다",
      "RECOVERY_ADMIN_ONLY",
      "BR-K12",
    );

    expect(recoveryFormErrorsFrom(error).fields).toEqual({
      staffId: "관리자 PIN만 다시 정할 수 있습니다",
    });
  });

  test("호스트의 칸 오류를 그 칸에 붙인다", () => {
    const error = new HostError(
      400,
      "형식이 맞지 않습니다",
      "INVALID_INPUT",
      null,
      [{ field: "pin", message: "숫자 6~8자리로 넣어 주세요" }],
    );

    expect(recoveryFormErrorsFrom(error).fields).toEqual({
      pin: "숫자 6~8자리로 넣어 주세요",
    });
  });
});
