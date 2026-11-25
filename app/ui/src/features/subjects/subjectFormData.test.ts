import { describe, expect, test } from "vitest";
import { HostError } from "../../api/client";
import type { SubjectResponse } from "../../api/generated/SubjectResponse";
import {
  EMPTY_SUBJECT_FORM,
  categoryNames,
  firstErrorField,
  subjectFormErrorsFrom,
  subjectFormValuesFrom,
  toSubjectRequest,
  usesCategories,
} from "./subjectFormData";

function subject(
  id: number,
  category: string | null,
  name = `과목${id}`,
): SubjectResponse {
  return { id, name, category };
}

describe("categoryNames", () => {
  test("분류 없음은 빼고, 겹치지 않게 가나다순", () => {
    const subjects = [
      subject(1, "평생교육"),
      subject(2, null),
      subject(3, "취미여가"),
      subject(4, "평생교육"),
    ];

    expect(categoryNames(subjects)).toEqual(["취미여가", "평생교육"]);
  });
});

describe("usesCategories", () => {
  test("분류를 쓴 과목이 없으면 false", () => {
    expect(usesCategories([subject(1, null), subject(2, null)])).toBe(false);
    expect(usesCategories([])).toBe(false);
  });

  test("하나라도 분류가 있으면 true", () => {
    expect(usesCategories([subject(1, null), subject(2, "평생교육")])).toBe(
      true,
    );
  });
});

describe("toSubjectRequest", () => {
  test("비운 분류는 null로 보낸다", () => {
    expect(toSubjectRequest({ name: "요가", category: "  " })).toEqual({
      name: "요가",
      category: null,
    });
  });

  test("채운 분류는 그대로 보낸다", () => {
    expect(
      toSubjectRequest({ ...EMPTY_SUBJECT_FORM, category: "취미여가" })
        .category,
    ).toBe("취미여가");
  });
});

describe("subjectFormValuesFrom", () => {
  test("분류가 없으면 빈 글자", () => {
    expect(subjectFormValuesFrom(subject(1, null, "요가"))).toEqual({
      name: "요가",
      category: "",
    });
  });
});

describe("subjectFormErrorsFrom", () => {
  test("이름이 겹치면 이름 칸에 붙인다 (BR-J1)", () => {
    const error = new HostError(
      409,
      "이미 있는 과목입니다. 띄어쓰기만 달라도 같은 과목으로 봅니다",
      "DUPLICATE_SUBJECT_NAME",
      "BR-J1",
    );

    expect(subjectFormErrorsFrom(error).fields).toEqual({
      name: "이미 있는 과목입니다. 띄어쓰기만 달라도 같은 과목으로 봅니다",
    });
  });

  test("강좌가 쓰는 과목을 지우려 하면 칸이 아닌 전체 문구로 (BR-J2)", () => {
    const message =
      "강좌가 쓰는 과목은 지울 수 없습니다. 지난 학기 강좌도 이 과목을 씁니다";
    const error = new HostError(409, message, "SUBJECT_IN_USE", "BR-J2");

    expect(subjectFormErrorsFrom(error)).toEqual({ fields: {}, form: message });
  });

  test("입력 오류는 과목 칸 이름으로 나눈다", () => {
    const error = new HostError(
      422,
      "입력을 확인해 주세요",
      "INVALID_INPUT",
      null,
      [{ field: "category", message: "분류 이름은 20자까지 씁니다" }],
    );

    expect(subjectFormErrorsFrom(error).fields).toEqual({
      category: "분류 이름은 20자까지 씁니다",
    });
  });
});

describe("firstErrorField", () => {
  test("화면에서 가장 위 칸을 고른다", () => {
    expect(firstErrorField({ category: "a", name: "b" })).toBe("name");
  });
});
