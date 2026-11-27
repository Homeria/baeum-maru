import { describe, expect, test } from "vitest";
import { HostError } from "../../../api/client";
import type { StaffResponse } from "../../../api/generated/StaffResponse";
import {
  EMPTY_STAFF_FORM,
  staffFormErrorsFrom,
  staffFormValuesFrom,
  staffStatusLabel,
  toCreateStaffRequest,
  toUpdateStaffRequest,
} from "./staffFormData";

const STAFF: StaffResponse = {
  id: 7,
  name: "임시1",
  role: "temp_staff",
  validUntil: "2027-02-28",
  disabled: false,
  locked: false,
};

describe("요청 본문", () => {
  test("새 직원: 비운 유효기간은 null, PIN 확인은 보내지 않는다", () => {
    const values = {
      ...EMPTY_STAFF_FORM,
      name: "직원1",
      pin: "1234",
      pinConfirm: "1234",
    };

    expect(toCreateStaffRequest(values)).toEqual({
      name: "직원1",
      role: "staff",
      validUntil: null,
      pin: "1234",
    });
  });

  test("고치기: 받은 직원으로 칸을 채우고 그대로 돌려보낸다", () => {
    const values = staffFormValuesFrom(STAFF);

    expect(values.pin).toBe("");
    expect(toUpdateStaffRequest(values)).toEqual({
      name: "임시1",
      role: "temp_staff",
      validUntil: "2027-02-28",
      disabled: false,
    });
  });
});

describe("staffFormErrorsFrom", () => {
  test("이름이 겹치면 이름 칸에", () => {
    const error = new HostError(
      409,
      "같은 이름의 직원이 있습니다",
      "DUPLICATE_STAFF_NAME",
      "BR-K2",
    );

    expect(staffFormErrorsFrom(error).fields).toEqual({
      name: "같은 이름의 직원이 있습니다",
    });
  });

  test("마지막 관리자는 전체 문구로", () => {
    const error = new HostError(
      409,
      "사용 중인 관리자가 한 명뿐이라",
      "LAST_ADMIN",
      "BR-K9",
    );

    expect(staffFormErrorsFrom(error).form).toBe(
      "사용 중인 관리자가 한 명뿐이라",
    );
  });
});

describe("staffStatusLabel", () => {
  test("사용 중지, 기간 끝남, 잠김, 사용 중 순서로 본다", () => {
    const today = "2027-03-01";

    expect(staffStatusLabel({ ...STAFF, disabled: true }, today)).toBe(
      "사용 중지",
    );
    expect(staffStatusLabel(STAFF, today)).toBe("기간 끝남");
    expect(staffStatusLabel({ ...STAFF, locked: true }, "2027-02-28")).toBe(
      "잠김",
    );
    expect(staffStatusLabel(STAFF, "2027-02-28")).toBe("사용 중");
  });
});
