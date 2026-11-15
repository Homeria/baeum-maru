import { describe, expect, test } from "vitest";
import { HostError } from "../../api/client";
import { firstErrorField, formErrorsFrom } from "./formErrors";

const FIELDS = ["name", "phoneNumber", "floor"] as const;

function invalid(fields: { field: string; message: string }[]): HostError {
  return new HostError(
    422,
    "입력을 확인해 주세요",
    "INVALID_INPUT",
    null,
    fields,
  );
}

describe("formErrorsFrom", () => {
  test("입력 오류는 칸마다 나눈다", () => {
    const error = invalid([
      { field: "name", message: "이름을 입력해 주세요" },
      { field: "floor", message: "층은 지하 2층부터 5층까지 고릅니다" },
    ]);

    expect(formErrorsFrom(error, FIELDS)).toEqual({
      fields: {
        name: "이름을 입력해 주세요",
        floor: "층은 지하 2층부터 5층까지 고릅니다",
      },
      form: null,
    });
  });

  test("한 칸에 오류가 둘이면 첫 번째만", () => {
    const error = invalid([
      { field: "name", message: "첫째" },
      { field: "name", message: "둘째" },
    ]);

    expect(formErrorsFrom(error, FIELDS).fields).toEqual({ name: "첫째" });
  });

  test("모르는 칸 이름은 버린다", () => {
    const error = invalid([{ field: "unknown", message: "?" }]);

    expect(formErrorsFrom(error, FIELDS).fields).toEqual({});
  });

  test("정해 둔 오류 종류는 그 칸에 붙인다", () => {
    const error = new HostError(
      409,
      "같은 건물에 같은 이름의 강의실이 있습니다",
      "DUPLICATE_ROOM_NAME",
      "BR-R1",
    );

    expect(
      formErrorsFrom(error, FIELDS, { DUPLICATE_ROOM_NAME: "name" }),
    ).toEqual({
      fields: { name: "같은 건물에 같은 이름의 강의실이 있습니다" },
      form: null,
    });
  });

  test("칸에 붙일 수 없는 오류는 폼 전체 문구로", () => {
    const error = new HostError(0, "호스트에 연결하지 못했습니다");

    expect(formErrorsFrom(error, FIELDS)).toEqual({
      fields: {},
      form: "호스트에 연결하지 못했습니다",
    });
  });

  test("호스트 오류가 아니면 일반 문구", () => {
    expect(formErrorsFrom(new Error("x"), FIELDS).form).toBe(
      "저장하지 못했습니다. 다시 해 주세요",
    );
  });
});

describe("firstErrorField", () => {
  test("화면에서 가장 위 칸을 고른다", () => {
    expect(firstErrorField({ floor: "a", phoneNumber: "b" }, FIELDS)).toBe(
      "phoneNumber",
    );
  });

  test("오류가 없으면 null", () => {
    expect(firstErrorField({}, FIELDS)).toBeNull();
  });
});
