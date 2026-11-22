import { describe, expect, test } from "vitest";
import { HostError } from "../../api/client";
import {
  EMPTY_PRIORITY_REASON_FORM,
  firstErrorField,
  priorityMethodLabel,
  priorityReasonFormErrorsFrom,
  priorityReasonFormValuesFrom,
  toPriorityReasonRequest,
} from "./priorityReasonFormData";

describe("priorityMethodLabel", () => {
  test("자리 보장의 화면 이름", () => {
    expect(priorityMethodLabel("guaranteed")).toBe("자리 보장");
  });
});

describe("입력 칸과 요청", () => {
  test("새 사유는 이름이 비고 방식은 자리 보장", () => {
    expect(toPriorityReasonRequest(EMPTY_PRIORITY_REASON_FORM)).toEqual({
      name: "",
      method: "guaranteed",
    });
  });

  test("고칠 사유로 칸을 채운다", () => {
    expect(
      priorityReasonFormValuesFrom({
        id: 1,
        name: "반장",
        method: "guaranteed",
      }),
    ).toEqual({ name: "반장", method: "guaranteed" });
  });
});

describe("priorityReasonFormErrorsFrom", () => {
  test("BR-S10 이름이 겹치면 이름 칸에 붙인다", () => {
    const message = "이미 있는 사유입니다";
    const error = new HostError(
      409,
      message,
      "DUPLICATE_PRIORITY_REASON_NAME",
      "BR-S10",
    );

    const errors = priorityReasonFormErrorsFrom(error);

    expect(errors.fields.name).toBe(message);
    expect(errors.form).toBeNull();
    expect(firstErrorField(errors.fields)).toBe("name");
  });

  test("BR-S10 신청에 붙었던 사유를 지우려 하면 칸이 아닌 문구로", () => {
    const message = "신청에 붙인 적이 있는 사유는 지울 수 없습니다";
    const error = new HostError(
      409,
      message,
      "PRIORITY_REASON_IN_USE",
      "BR-S10",
    );

    const errors = priorityReasonFormErrorsFrom(error);

    expect(errors.form).toBe(message);
    expect(firstErrorField(errors.fields)).toBeNull();
  });
});
