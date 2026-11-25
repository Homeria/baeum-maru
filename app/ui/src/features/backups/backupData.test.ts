import { describe, expect, test } from "vitest";
import type { SecondLocationResponse } from "../../api/generated/SecondLocationResponse";
import {
  formatSize,
  formatTakenAt,
  kindText,
  previewLines,
  restoreOutcomeText,
  secondLocationWarning,
} from "./backupData";

const TODAY = new Date(2026, 10, 23, 10, 0);

function second(
  overrides: Partial<SecondLocationResponse>,
): SecondLocationResponse {
  return {
    path: "E:\\백업",
    reachable: true,
    lastCopiedAt: "2026-11-23T09:00:00",
    changeable: true,
    ...overrides,
  };
}

describe("kindText", () => {
  test("추첨·마감 직전은 강좌 이름을 붙인다", () => {
    expect(
      kindText({
        kind: "before_lottery",
        courseId: 3,
        courseName: "요가 초급",
      }),
    ).toBe("추첨 직전 · 요가 초급");
    expect(kindText({ kind: "daily", courseId: null, courseName: null })).toBe(
      "하루 한 번",
    );
  });

  test("강좌가 없어졌으면 id를 보여 준다", () => {
    expect(
      kindText({ kind: "before_close", courseId: 7, courseName: null }),
    ).toBe("선발 마감 직전 · 지운 강좌(7)");
  });
});

describe("formatTakenAt", () => {
  test("초를 떼고 날짜와 시각 사이를 띄운다", () => {
    expect(formatTakenAt("2026-11-23T09:05:30")).toBe("2026-11-23 09:05");
  });
});

describe("formatSize", () => {
  test("크기에 맞는 단위를 쓴다", () => {
    expect(formatSize(512)).toBe("512 B");
    expect(formatSize(1536)).toBe("1.5 KB");
    expect(formatSize(3 * 1024 * 1024)).toBe("3.0 MB");
  });
});

describe("secondLocationWarning", () => {
  test("꺼져 있거나 최근에 복사했으면 경고하지 않는다", () => {
    expect(secondLocationWarning(second({ path: null }), TODAY)).toBeNull();
    expect(secondLocationWarning(second({}), TODAY)).toBeNull();
    expect(
      secondLocationWarning(
        second({ lastCopiedAt: "2026-11-20T09:00:00" }),
        TODAY,
      ),
    ).toBeNull();
  });

  test("오래 복사하지 못했으면 며칠째인지 알린다", () => {
    expect(
      secondLocationWarning(
        second({ lastCopiedAt: "2026-11-18T09:00:00" }),
        TODAY,
      ),
    ).toBe("두 번째 위치에 5일째 복사하지 못했습니다");
  });

  test("연결되지 않았거나 복사한 적이 없으면 알린다", () => {
    expect(
      secondLocationWarning(second({ reachable: false }), TODAY),
    ).toContain("연결되지 않았습니다");
    expect(secondLocationWarning(second({ lastCopiedAt: null }), TODAY)).toBe(
      "두 번째 위치에 아직 복사한 백업이 없습니다",
    );
  });
});

describe("previewLines", () => {
  test("백업 시각과 종류, 학기, 회원·신청 수를 보여 준다", () => {
    expect(
      previewLines({
        takenAt: "2026-11-20T09:00:00",
        kind: "daily",
        termNames: ["2026년 2학기", "2027년 1학기"],
        memberCount: 120,
        applicationCount: 340,
      }),
    ).toEqual([
      "백업: 2026-11-20 09:00 · 하루 한 번",
      "학기: 2026년 2학기, 2027년 1학기",
      "회원 120명, 신청 340건",
    ]);
  });

  test("이름을 바꾼 파일과 빈 데이터", () => {
    expect(
      previewLines({
        takenAt: null,
        kind: null,
        termNames: [],
        memberCount: 0,
        applicationCount: 0,
      }),
    ).toEqual([
      "백업: 알 수 없음 (파일 이름이 바뀜)",
      "학기: 없음",
      "회원 0명, 신청 0건",
    ]);
  });
});

describe("restoreOutcomeText", () => {
  test("되돌렸는지 못 했는지 알린다", () => {
    expect(
      restoreOutcomeText({ at: "2026-11-23T10:00:00", restored: true }),
    ).toBe("2026-11-23 10:00에 백업으로 되돌렸습니다");
    expect(
      restoreOutcomeText({ at: "2026-11-23T10:00:00", restored: false }),
    ).toContain("그 전 데이터를 그대로 쓰고 있습니다");
  });
});
