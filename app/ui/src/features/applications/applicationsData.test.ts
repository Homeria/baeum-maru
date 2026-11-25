import { describe, expect, test } from "vitest";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { TermResponse } from "../../api/generated/TermResponse";
import {
  additionalLabel,
  isTermEnded,
  priorityRequests,
  submittedNotice,
  toggleChoice,
  toggleId,
  togglePriority,
  violationMessage,
} from "./applicationsData";

describe("toggleChoice", () => {
  test("없으면 뒤에 넣고, 있으면 뺀다", () => {
    expect(toggleChoice([3, 1], 2)).toEqual([3, 1, 2]);
    expect(toggleChoice([3, 1, 2], 1)).toEqual([3, 2]);
  });
});

describe("violationMessage", () => {
  const course = (id: number, subjectName: string, level: string | null) =>
    ({
      id,
      termId: 1,
      subjectId: id,
      subjectName,
      level,
      section: null,
      capacity: 20,
      schedules: [],
    }) satisfies CourseResponse;
  const courses = [course(1, "요가", "초급"), course(3, "한글교실", null)];

  test("어긴 강좌, 규칙, 부딪힌 강좌를 보여 준다", () => {
    const message = violationMessage(
      {
        courseId: 3,
        rule: "no_time_overlap",
        otherCourseId: 1,
        canOverride: true,
      },
      courses,
    );

    expect(message).toBe("한글교실: 시간 겹침 (요가 초급)");
  });

  test("같은 강좌 두 번은 이미 신청한 강좌라고 쓴다", () => {
    const message = violationMessage(
      {
        courseId: 1,
        rule: "no_duplicate",
        otherCourseId: 1,
        canOverride: false,
      },
      courses,
    );

    expect(message).toBe("요가 초급: 이미 신청한 강좌");
  });
});

describe("toggleId", () => {
  test("없으면 넣고 작은 id부터 둔다", () => {
    expect(toggleId([3], 1)).toEqual([1, 3]);
  });

  test("있으면 뺀다", () => {
    expect(toggleId([1, 3], 3)).toEqual([1]);
  });
});

describe("togglePriority", () => {
  test("사유가 다 빠진 강좌는 지운다", () => {
    const once = togglePriority(new Map(), 10, 1);
    const twice = togglePriority(once, 10, 1);

    expect(once.get(10)).toEqual([1]);
    expect(twice.has(10)).toBe(false);
  });
});

describe("priorityRequests", () => {
  test("BR-S12 고른 강좌의 사유만, 고른 순서로", () => {
    const priorities = new Map([
      [10, [2, 1]],
      [20, [1]],
      [30, [1]],
    ]);

    expect(priorityRequests([20, 10], priorities)).toEqual([
      { courseId: 20, reasonIds: [1] },
      { courseId: 10, reasonIds: [2, 1] },
    ]);
  });

  test("사유가 없는 강좌는 싣지 않는다", () => {
    expect(priorityRequests([10], new Map())).toEqual([]);
  });
});

describe("추가 신청 (BR-X1, BR-X2)", () => {
  const course = (id: number, subjectName: string): CourseResponse =>
    ({ id, subjectName, level: null, section: null }) as CourseResponse;
  const courses = [course(1, "요가"), course(2, "한글교실")];
  const waitlisted = (waitlistNumber: number) =>
    ({ kind: "waitlisted", waitlistNumber }) as const;

  test("결과 글자", () => {
    expect(additionalLabel({ kind: "selected" })).toBe("바로 당첨");
    expect(additionalLabel(waitlisted(3))).toBe("대기 3번째");
  });

  test("학기 종료일 당일까지 받는다", () => {
    const term = { endsOn: "2027-06-30" } as TermResponse;

    expect(isTermEnded(term, "2027-06-30")).toBe(false);
    expect(isTermEnded(term, "2027-07-01")).toBe(true);
  });

  test("보통 신청은 건수, 추가 신청은 강좌마다 결과를 알린다", () => {
    const lines = submittedNotice(
      [
        { id: 10, courseId: 2, additional: null },
        { id: 11, courseId: 1, additional: waitlisted(4) },
      ],
      new Map([[1, waitlisted(4)]]),
      courses,
    );

    expect(lines).toEqual(["1건 신청했습니다", "요가: 추가 신청, 대기 4번째"]);
  });

  test("바로 당첨으로 보았는데 대기로 받았으면 자리가 찼다고 알린다", () => {
    const lines = submittedNotice(
      [{ id: 11, courseId: 1, additional: waitlisted(1) }],
      new Map([[1, { kind: "selected" } as const]]),
      courses,
    );

    expect(lines).toEqual([
      "요가: 자리가 방금 찼습니다. 대기 1번째로 받았습니다",
    ]);
  });
});
