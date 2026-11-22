import { describe, expect, test } from "vitest";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import {
  countLabel,
  countsByCourse,
  priorityRequests,
  toggleChoice,
  toggleId,
  togglePriority,
  violationMessage,
} from "./applicationsData";

describe("countsByCourse", () => {
  test("강좌 id로 인원을 찾고, 없는 강좌는 찾지 못한다", () => {
    const counts = countsByCourse([
      { courseId: 1, count: 12 },
      { courseId: 3, count: 1 },
    ]);

    expect(counts.get(1)).toBe(12);
    expect(counts.get(3)).toBe(1);
    expect(counts.get(2)).toBeUndefined();
  });
});

describe("countLabel", () => {
  test("신청 인원/정원으로 보여 준다", () => {
    expect(countLabel(12, 20)).toBe("12/20");
  });
});

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
