import type { RoomRequest } from "../../api/generated/RoomRequest";
import type { RoomResponse } from "../../api/generated/RoomResponse";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";

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

/** 고칠 강의실로 입력 칸을 채운다. 층이 없으면 `""`. */
export function roomFormValuesFrom(room: RoomResponse): RoomFormValues {
  return {
    name: room.name,
    building: room.building,
    floor: room.floor === null ? "" : String(room.floor),
  };
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
export type RoomFormErrors = FormErrors<RoomField>;

/** 이름이 겹치면(BR-R1) 이름 칸에 붙인다. */
export function roomFormErrorsFrom(error: unknown): RoomFormErrors {
  return formErrorsFrom(error, ROOM_FIELDS, { DUPLICATE_ROOM_NAME: "name" });
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<RoomField, string>>,
): RoomField | null {
  return firstOf(fields, ROOM_FIELDS);
}
