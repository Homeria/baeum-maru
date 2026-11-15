import { HostError } from "../../api/client";
import type { CourseRequest } from "../../api/generated/CourseRequest";
import type { CourseResponse } from "../../api/generated/CourseResponse";
import type { ScheduleResponse } from "../../api/generated/ScheduleResponse";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";

/** 요일 이름. 번호(1=월 … 7=일)에서 1을 뺀 자리. */
export const WEEKDAYS = ["월", "화", "수", "목", "금", "토", "일"] as const;

/** 요일 번호(1~7)의 이름. */
export function weekdayLabel(weekday: number): string {
  return WEEKDAYS[weekday - 1] ?? "?";
}

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. */
export type CourseField = keyof CourseRequest;

/** 수업 시간 한 줄의 입력 값. `key`는 줄을 지워도 다른 줄의 입력이 섞이지 않게 붙이는 번호. */
export interface ScheduleRowValues {
  key: number;
  /** "1"~"7" */
  weekday: string;
  /** HH:MM, 비웠으면 `""` */
  start: string;
  end: string;
  /** 강의실 id, 고르지 않았으면 `""` */
  roomId: string;
}

/** 입력 칸에 들어 있는 값 그대로. 고르지 않았거나 비웠으면 `""`. */
export interface CourseFormValues {
  subjectId: string;
  level: string;
  section: string;
  capacity: string;
  schedules: ScheduleRowValues[];
}

const EMPTY_ROW: Omit<ScheduleRowValues, "key"> = {
  weekday: "1",
  start: "",
  end: "",
  roomId: "",
};

export const EMPTY_COURSE_FORM: CourseFormValues = {
  subjectId: "",
  level: "",
  section: "",
  capacity: "",
  schedules: [{ key: 1, ...EMPTY_ROW }],
};

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const COURSE_FIELDS: readonly CourseField[] = [
  "subjectId",
  "level",
  "section",
  "capacity",
  "schedules",
];

/**
 * 수업 시간 줄을 하나 더한다. 앞 줄의 시각과 강의실을 그대로 쓰고 요일만 다음 날로.
 *
 * 월·수처럼 같은 시간에 요일만 다른 강좌가 많아서다. 겹치지 않게 요일은 바꿔 둔다.
 */
export function addScheduleRow(
  rows: readonly ScheduleRowValues[],
): ScheduleRowValues[] {
  const key = Math.max(0, ...rows.map((row) => row.key)) + 1;
  const last = rows.at(-1);
  const next = last
    ? { ...last, key, weekday: String((Number(last.weekday) % 7) + 1) }
    : { key, ...EMPTY_ROW };
  return [...rows, next];
}

/** 고칠 강좌로 입력 칸을 채운다. */
export function courseFormValuesFrom(course: CourseResponse): CourseFormValues {
  return {
    subjectId: String(course.subjectId),
    level: course.level ?? "",
    section: course.section ?? "",
    capacity: String(course.capacity),
    schedules: course.schedules.map((schedule, index) => ({
      key: index + 1,
      weekday: String(schedule.weekday),
      start: schedule.start,
      end: schedule.end,
      roomId: schedule.room ? String(schedule.room.id) : "",
    })),
  };
}

/** 입력 칸 값을 요청 본문으로. 고르지 않은 값은 `null`. 확인은 호스트가 한다. */
export function toCourseRequest(values: CourseFormValues): CourseRequest {
  return {
    subjectId: values.subjectId === "" ? null : Number(values.subjectId),
    level: values.level,
    section: values.section,
    capacity: values.capacity === "" ? null : Number(values.capacity),
    schedules: values.schedules.map((row) => ({
      weekday: Number(row.weekday),
      start: row.start,
      end: row.end,
      roomId: row.roomId === "" ? null : Number(row.roomId),
    })),
  };
}

/** 강좌 이름. 예: `요가 초급 1반`. 지우기 확인과 알림에 쓴다. */
export function courseName(
  course: Pick<CourseResponse, "subjectName" | "level" | "section">,
): string {
  return [course.subjectName, course.level, course.section]
    .filter((part) => part !== null && part !== "")
    .join(" ");
}

/** 강의실 이름. 건물이 여럿일 때만 건물을 붙인다. 예: `본관 201호` */
export function roomLabel(
  room: { building: string; name: string },
  showBuilding: boolean,
): string {
  return showBuilding ? `${room.building} ${room.name}` : room.name;
}

/** 목록 한 줄에 보여 줄 수업 시간과 강의실. */
export interface ScheduleSummary {
  /** 예: `월·수 10:00~11:00, 금 14:00~15:00` */
  times: string;
  /** 모든 줄이 같은 강의실이면 그 이름, 강의실이 없으면 `""`. 줄마다 다르면 `times`에 붙인다. */
  room: string;
}

/**
 * 수업 시간을 짧게 줄인다. 시각이 같은 요일은 `월·수`처럼 묶는다.
 *
 * 줄은 호스트가 요일, 시작 시각 순서로 준다. 묶음 순서는 처음 나온 순서를 따른다.
 */
export function scheduleSummary(
  schedules: readonly ScheduleResponse[],
  showBuilding: boolean,
): ScheduleSummary {
  const firstRoomId = schedules[0]?.room?.id ?? null;
  const sameRoom = schedules.every(
    (schedule) => (schedule.room?.id ?? null) === firstRoomId,
  );
  const placeOf = (schedule: ScheduleResponse) =>
    schedule.room ? roomLabel(schedule.room, showBuilding) : "강의실 없음";

  const groups = new Map<string, { days: string[]; text: string }>();
  for (const schedule of schedules) {
    const time = `${schedule.start}~${schedule.end}`;
    const text = sameRoom ? time : `${time} ${placeOf(schedule)}`;
    const group = groups.get(text) ?? { days: [], text };
    group.days.push(weekdayLabel(schedule.weekday));
    groups.set(text, group);
  }
  const times = [...groups.values()]
    .map((group) => `${group.days.join("·")} ${group.text}`)
    .join(", ");

  const first = schedules[0]?.room;
  const room = !sameRoom
    ? "요일마다 다름"
    : first
      ? roomLabel(first, showBuilding)
      : "";
  return { times, room };
}

export type CourseFormErrors = FormErrors<CourseField>;

/**
 * 저장·지우기 실패를 칸에 나눠 붙인다.
 *
 * 수업 시간 오류는 줄마다 여러 개일 수 있어 모두 줄을 바꿔 이어 붙인다. 문장에 줄 번호가 있다.
 * 같은 강좌 겹침(BR-T3)은 과목·수준·반 세 칸에 걸쳐 있어 칸이 아닌 전체 문구로.
 */
export function courseFormErrorsFrom(error: unknown): CourseFormErrors {
  const errors = formErrorsFrom(error, COURSE_FIELDS);
  if (error instanceof HostError && error.code === "INVALID_INPUT") {
    const messages = error.fields
      .filter((field) => field.field === "schedules")
      .map((field) => field.message);
    if (messages.length > 0) {
      errors.fields.schedules = messages.join("\n");
    }
  }
  return errors;
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<CourseField, string>>,
): CourseField | null {
  return firstOf(fields, COURSE_FIELDS);
}
