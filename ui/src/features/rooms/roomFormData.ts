import { HostError } from "../../api/client";
import type { RoomRequest } from "../../api/generated/RoomRequest";
import type { RoomResponse } from "../../api/generated/RoomResponse";

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. */
export type RoomField = keyof RoomRequest;

/** 입력 칸에 들어 있는 글자 그대로. 층을 고르지 않았으면 `""`. */
export interface RoomFormValues {
  name: string;
  building: string;
  floor: string;
}

/** 강의실이 하나도 없을 때 건물 칸에 미리 넣는 이름. */
export const DEFAULT_BUILDING = "본관";

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const ROOM_FIELDS: readonly RoomField[] = ["name", "building", "floor"];

/**
 * 등록 칸의 처음 건물. 가장 최근에 등록한 강의실의 건물이고, 강의실이 없으면 본관.
 *
 * 기관 대부분은 건물이 하나라서 직원이 건물을 칠 일이 거의 없게 한다.
 */
export function lastUsedBuilding(rooms: readonly RoomResponse[]): string {
  const latest = rooms.reduce<RoomResponse | null>(
    (found, room) => (found === null || room.id > found.id ? room : found),
    null,
  );
  return latest?.building ?? DEFAULT_BUILDING;
}

/** 건물 칸 자동완성에 보여 줄 이름. 겹치지 않게 가나다순. */
export function buildingNames(rooms: readonly RoomResponse[]): string[] {
  return [...new Set(rooms.map((room) => room.building))].sort((a, b) =>
    a.localeCompare(b, "ko"),
  );
}

/** 건물이 둘 이상인지. 하나뿐이면 목록에서 건물을 숨긴다. */
export function hasSeveralBuildings(rooms: readonly RoomResponse[]): boolean {
  return new Set(rooms.map((room) => room.building)).size > 1;
}

/** 입력 칸 값을 호스트 요청으로 바꾼다. 층을 고르지 않았으면 `null`. 확인은 호스트가 한다. */
export function toRoomRequest(values: RoomFormValues): RoomRequest {
  return {
    name: values.name,
    building: values.building,
    floor: values.floor === "" ? null : Number(values.floor),
  };
}

/** 저장 실패를 칸별 문구와 칸에 붙일 수 없는 문구로 나눈다. */
export interface RoomFormErrors {
  fields: Partial<Record<RoomField, string>>;
  form: string | null;
}

export function roomFormErrorsFrom(error: unknown): RoomFormErrors {
  if (!(error instanceof HostError)) {
    return { fields: {}, form: "저장하지 못했습니다. 다시 해 주세요" };
  }
  if (error.code === "DUPLICATE_ROOM_NAME") {
    return { fields: { name: error.message }, form: null };
  }
  if (error.code === "INVALID_INPUT") {
    const fields: Partial<Record<RoomField, string>> = {};
    for (const { field, message } of error.fields) {
      // 한 칸에 오류가 둘이면 첫 번째만
      if (isRoomField(field) && fields[field] === undefined) {
        fields[field] = message;
      }
    }
    return { fields, form: null };
  }
  return { fields: {}, form: error.message };
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<RoomField, string>>,
): RoomField | null {
  return ROOM_FIELDS.find((field) => fields[field] !== undefined) ?? null;
}

function isRoomField(field: string): field is RoomField {
  return (ROOM_FIELDS as readonly string[]).includes(field);
}
