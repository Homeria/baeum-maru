import { describe, expect, test } from "vitest";
import { HostError } from "../../../api/client";
import {
  EMPTY_MEMBER_FORM,
  firstErrorField,
  memberFormErrorsFrom,
  memberFormValuesFrom,
  toMemberRequest,
} from "./memberFormData";

describe("toMemberRequest", () => {
  test("비운 선택 칸은 null로 보낸다", () => {
    const request = toMemberRequest({
      ...EMPTY_MEMBER_FORM,
      memberNo: "00001",
      name: "회원1",
      phoneNumber: "010-0000-0001",
    });

    expect(request).toEqual({
      memberNo: "00001",
      name: "회원1",
      phoneNumber: "010-0000-0001",
      birthDate: null,
      gender: null,
    });
  });

  test("채운 선택 칸은 그대로 보낸다", () => {
    const request = toMemberRequest({
      ...EMPTY_MEMBER_FORM,
      birthDate: "1950-03-01",
      gender: "female",
    });

    expect(request.birthDate).toBe("1950-03-01");
    expect(request.gender).toBe("female");
  });
});

describe("memberFormErrorsFrom", () => {
  test("입력 오류는 칸마다 나눈다", () => {
    const error = new HostError(
      422,
      "입력을 확인해 주세요",
      "INVALID_INPUT",
      null,
      [
        { field: "name", message: "이름을 입력해 주세요" },
        {
          field: "phoneNumber",
          message: "연락처는 0으로 시작하는 숫자 9~11자리입니다",
        },
      ],
    );

    const errors = memberFormErrorsFrom(error);

    expect(errors.fields).toEqual({
      name: "이름을 입력해 주세요",
      phoneNumber: "연락처는 0으로 시작하는 숫자 9~11자리입니다",
    });
    expect(errors.form).toBeNull();
  });

  test("회원번호가 겹치면 회원번호 칸에 붙인다", () => {
    const error = new HostError(
      409,
      "이미 있는 회원번호입니다",
      "DUPLICATE_MEMBER_NO",
      "BR-M2",
    );

    expect(memberFormErrorsFrom(error).fields).toEqual({
      memberNo: "이미 있는 회원번호입니다",
    });
  });
});

describe("firstErrorField", () => {
  test("화면에서 가장 위 칸을 고른다", () => {
    expect(firstErrorField({ phoneNumber: "a", name: "b" })).toBe("name");
  });
});

describe("memberFormValuesFrom", () => {
  const member = {
    id: 1,
    memberNo: "00001",
    name: "회원1",
    phoneNumber: "01000000001",
    birthDate: null,
    gender: null,
    createdAt: "2026-11-15T00:00:00.000Z",
    updatedAt: "2026-11-15T00:00:00.000Z",
  };

  test("연락처에 하이픈을 넣고 빈 선택 칸은 빈 글자로", () => {
    expect(memberFormValuesFrom(member)).toEqual({
      memberNo: "00001",
      name: "회원1",
      phoneNumber: "010-0000-0001",
      birthDate: "",
      gender: "",
    });
  });

  test("채운 선택 칸은 그대로", () => {
    const values = memberFormValuesFrom({
      ...member,
      birthDate: "1950-03-01",
      gender: "male",
    });

    expect(values.birthDate).toBe("1950-03-01");
    expect(values.gender).toBe("male");
  });
});
