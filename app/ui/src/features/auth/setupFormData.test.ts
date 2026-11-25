import { describe, expect, it } from "vitest";
import { HostError } from "../../api/client";
import {
  checkPinConfirm,
  firstErrorField,
  setupFormErrorsFrom,
  toSetupRequest,
} from "./setupFormData";

describe("toSetupRequest", () => {
  it("PIN 확인 칸은 보내지 않는다", () => {
    const request = toSetupRequest({
      name: "관리자",
      pin: "123456",
      pinConfirm: "123456",
    });

    expect(request).toEqual({ name: "관리자", pin: "123456" });
  });
});

describe("checkPinConfirm", () => {
  it("두 PIN이 같으면 오류가 없다", () => {
    expect(
      checkPinConfirm({ name: "", pin: "123456", pinConfirm: "123456" }),
    ).toBeNull();
  });

  it("다르면 PIN 확인 칸에 붙인다", () => {
    const errors = checkPinConfirm({
      name: "",
      pin: "123456",
      pinConfirm: "123465",
    });

    expect(errors?.fields).toEqual({ pinConfirm: "위에 넣은 PIN과 다릅니다" });
  });
});

describe("setupFormErrorsFrom", () => {
  it("호스트의 칸 오류를 이름·PIN 칸에 붙인다", () => {
    const error = new HostError(
      422,
      "입력을 확인해 주세요",
      "INVALID_INPUT",
      null,
      [
        { field: "name", message: "이름을 입력해 주세요" },
        { field: "pin", message: "관리자 PIN은 6자리 이상입니다" },
      ],
    );

    const errors = setupFormErrorsFrom(error);

    expect(errors.fields).toEqual({
      name: "이름을 입력해 주세요",
      pin: "관리자 PIN은 6자리 이상입니다",
    });
    expect(firstErrorField(errors.fields)).toBe("name");
  });

  it("이미 설정을 마쳤으면 전체 문구로 보여 준다", () => {
    const error = new HostError(
      409,
      "이미 처음 설정을 마쳤습니다. 관리자 계정으로 로그인해 주세요",
      "SETUP_ALREADY_DONE",
      "BR-K11",
    );

    expect(setupFormErrorsFrom(error).form).toBe(
      "이미 처음 설정을 마쳤습니다. 관리자 계정으로 로그인해 주세요",
    );
  });
});
