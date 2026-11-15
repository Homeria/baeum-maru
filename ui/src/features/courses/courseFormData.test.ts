import { describe, expect, test } from "vitest";
import { HostError } from "../../api/client";
import type { ScheduleResponse } from "../../api/generated/ScheduleResponse";
import {
  EMPTY_COURSE_FORM,
  addScheduleRow,
  courseFormErrorsFrom,
  courseFormValuesFrom,
  courseName,
  scheduleSummary,
  toCourseRequest,
} from "./courseFormData";

const ROOM_201 = { id: 3, building: "본관", name: "201호" };
const ROOM_302 = { id: 4, building: "별관", name: "302호" };

function schedule(
  weekday: number,
  start: string,
  end: string,
  room: ScheduleResponse["room"] = null,
): ScheduleResponse {
  return { weekday, start, end, room };
}

describe("addScheduleRow", () => {
  test("앞 줄의 시각과 강의실을 쓰고 요일은 다음 날로", () => {
    const rows = [
      { key: 1, weekday: "1", start: "10:00", end: "11:00", roomId: "3" },
    ];

    expect(addScheduleRow(rows)[1]).toEqual({
      key: 2,
      weekday: "2",
      start: "10:00",
      end: "11:00",
      roomId: "3",
    });
  });

  test("일요일 다음은 월요일", () => {
    const rows = [{ key: 5, weekday: "7", start: "", end: "", roomId: "" }];

    expect(addScheduleRow(rows)[1]).toMatchObject({ key: 6, weekday: "1" });
  });
});

describe("toCourseRequest", () => {
  test("고르지 않은 값은 null, 숫자 칸은 숫자로", () => {
    expect(toCourseRequest(EMPTY_COURSE_FORM)).toEqual({
      subjectId: null,
      level: "",
      section: "",
      capacity: null,
      schedules: [{ weekday: 1, start: "", end: "", roomId: null }],
    });
  });

  test("고칠 강좌로 채운 값을 그대로 보내면 같은 내용", () => {
    const values = courseFormValuesFrom({
      id: 1,
      termId: 1,
      subjectId: 2,
      subjectName: "요가",
      level: "초급",
      section: null,
      capacity: 20,
      schedules: [schedule(1, "10:00", "11:00", ROOM_201)],
    });

    expect(toCourseRequest(values)).toEqual({
      subjectId: 2,
      level: "초급",
      section: "",
      capacity: 20,
      schedules: [{ weekday: 1, start: "10:00", end: "11:00", roomId: 3 }],
    });
  });
});

describe("courseName", () => {
  test("과목, 수준, 반 중 있는 것만", () => {
    expect(
      courseName({ subjectName: "요가", level: null, section: "1반" }),
    ).toBe("요가 1반");
  });
});

describe("scheduleSummary", () => {
  test("시각이 같은 요일을 묶고, 강의실이 모두 같으면 한 번만", () => {
    const schedules = [
      schedule(1, "10:00", "11:00", ROOM_201),
      schedule(3, "10:00", "11:00", ROOM_201),
      schedule(5, "14:00", "15:00", ROOM_201),
    ];

    expect(scheduleSummary(schedules, false)).toEqual({
      times: "월·수 10:00~11:00, 금 14:00~15:00",
      room: "201호",
    });
  });

  test("강의실이 줄마다 다르면 시각 옆에 붙인다", () => {
    const schedules = [
      schedule(1, "10:00", "11:00", ROOM_201),
      schedule(3, "10:00", "11:00", ROOM_302),
    ];

    expect(scheduleSummary(schedules, true)).toEqual({
      times: "월 10:00~11:00 본관 201호, 수 10:00~11:00 별관 302호",
      room: "요일마다 다름",
    });
  });

  test("강의실이 없으면 빈 글자", () => {
    expect(scheduleSummary([schedule(2, "09:00", "09:50")], false)).toEqual({
      times: "화 09:00~09:50",
      room: "",
    });
  });
});

describe("courseFormErrorsFrom", () => {
  test("수업 시간 오류는 모두 줄을 바꿔 붙인다", () => {
    const error = new HostError(
      422,
      "입력을 확인해 주세요",
      "INVALID_INPUT",
      null,
      [
        { field: "capacity", message: "정원을 입력해 주세요" },
        {
          field: "schedules",
          message: "1번째 수업 시간: 시작·종료 시각을 입력해 주세요",
        },
        {
          field: "schedules",
          message: "2번째 수업 시간: 종료 시각은 시작 시각보다 늦어야 합니다",
        },
      ],
    );

    expect(courseFormErrorsFrom(error).fields).toEqual({
      capacity: "정원을 입력해 주세요",
      schedules:
        "1번째 수업 시간: 시작·종료 시각을 입력해 주세요\n2번째 수업 시간: 종료 시각은 시작 시각보다 늦어야 합니다",
    });
  });

  test("같은 강좌 겹침은 전체 문구로 (BR-T3)", () => {
    const message = "이 학기에 과목·수준·반이 모두 같은 강좌가 있습니다";
    const error = new HostError(409, message, "DUPLICATE_COURSE", "BR-T3");

    expect(courseFormErrorsFrom(error)).toEqual({ fields: {}, form: message });
  });
});
