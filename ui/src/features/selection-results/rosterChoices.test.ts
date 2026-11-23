import { describe, expect, test } from "vitest";
import {
  defaultChoices,
  exportRequest,
  rememberChoices,
  rememberedChoices,
  rememberedKind,
  rosterFileName,
  toggle,
  type ChoiceStorage,
} from "./rosterChoices";

function memoryStorage(initial: string | null = null): ChoiceStorage {
  let value = initial;
  return {
    getItem: () => value,
    setItem: (_key, next) => {
      value = next;
    },
  };
}

describe("기억한 선택", () => {
  test("처음 쓰는 PC는 처음 선택", () => {
    const storage = memoryStorage();

    expect(rememberedChoices(storage, "raw")).toEqual(defaultChoices("raw"));
    expect(rememberedChoices(storage, "posted_by_member").mask).toEqual([
      "name",
    ]);
  });

  test("BR-P9: 연락처·생년월일·성별은 기억했어도 끈 채로 연다", () => {
    const storage = memoryStorage();
    rememberChoices(storage, "raw", {
      include: ["phone_number", "gender"],
      mask: ["phone_number"],
      maskChar: "x",
    });

    expect(rememberedChoices(storage, "raw")).toEqual({
      include: [],
      mask: ["phone_number"],
      maskChar: "x",
    });
  });

  test("명단 종류마다 따로 기억하고, 마지막 종류를 기억한다", () => {
    const storage = memoryStorage();
    rememberChoices(storage, "raw", {
      ...defaultChoices("raw"),
      maskChar: "x",
    });
    rememberChoices(storage, "posted_by_course", {
      include: [],
      mask: [],
      maskChar: "asterisk",
    });

    expect(rememberedChoices(storage, "raw").maskChar).toBe("x");
    expect(rememberedChoices(storage, "posted_by_course").mask).toEqual([]);
    expect(rememberedKind(storage, ["posted_by_course", "raw"])).toBe(
      "posted_by_course",
    );
  });

  test("고를 수 없는 종류를 기억했으면 고를 수 있는 첫 종류", () => {
    const storage = memoryStorage(JSON.stringify({ kind: "raw" }));

    expect(rememberedKind(storage, ["posted_by_course"])).toBe(
      "posted_by_course",
    );
  });

  test("예전에 기억한 원본의 정원·우선 선택은 버린다", () => {
    const old = memoryStorage(
      JSON.stringify({
        choices: {
          raw: {
            include: ["capacity", "waitlist_number", "guaranteed"],
            mask: ["name"],
            maskChar: "x",
          },
        },
      }),
    );

    expect(rememberedChoices(old, "raw")).toEqual({
      include: [],
      mask: ["name"],
      maskChar: "x",
    });
  });

  test("고장 났거나 규칙에 없는 값은 버린다", () => {
    expect(rememberedChoices(memoryStorage("{고장"), "raw")).toEqual(
      defaultChoices("raw"),
    );
    const odd = memoryStorage(
      JSON.stringify({
        choices: {
          posted_by_member: {
            include: ["guaranteed"],
            mask: ["gender", "member_no"],
            maskChar: "?",
          },
        },
      }),
    );

    expect(rememberedChoices(odd, "posted_by_member")).toEqual({
      include: [],
      mask: ["member_no"],
      maskChar: "circle",
    });
  });
});

describe("요청", () => {
  test("넣지 않은 칸의 가림은 보내지 않는다", () => {
    expect(
      exportRequest("raw", {
        include: ["gender"],
        mask: ["name", "phone_number"],
        maskChar: "circle",
      }),
    ).toEqual({
      kind: "raw",
      include: ["gender"],
      mask: ["name"],
      maskChar: "circle",
    });
  });

  test("켜고 끄면 정한 순서를 지킨다", () => {
    const order = ["phone_number", "birth_date", "gender"] as const;

    expect(toggle(["gender"], "phone_number", true, order)).toEqual([
      "phone_number",
      "gender",
    ]);
    expect(toggle(["phone_number", "gender"], "gender", false, order)).toEqual([
      "phone_number",
    ]);
  });

  test("파일 이름은 학기와 명단 종류", () => {
    expect(rosterFileName("2027년 1학기", "posted_by_course")).toBe(
      "2027년 1학기 게시용 강좌별.xlsx",
    );
  });
});
