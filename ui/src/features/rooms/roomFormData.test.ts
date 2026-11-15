import { describe, expect, test } from "vitest";
import { HostError } from "../../api/client";
import type { RoomResponse } from "../../api/generated/RoomResponse";
import {
  DEFAULT_BUILDING,
  buildingNames,
  firstErrorField,
  hasSeveralBuildings,
  lastUsedBuilding,
  roomFormErrorsFrom,
  toRoomRequest,
} from "./roomFormData";

function room(
  id: number,
  building: string,
  name = `강의실${id}`,
): RoomResponse {
  return { id, building, floor: null, name };
}

describe("lastUsedBuilding", () => {
  test("강의실이 없으면 본관", () => {
    expect(lastUsedBuilding([])).toBe(DEFAULT_BUILDING);
  });

  test("가장 최근에 등록한 강의실의 건물", () => {
    // 목록은 건물 순서라 id 순서와 다르다
    const rooms = [room(3, "별관"), room(1, "본관"), room(2, "본관")];

    expect(lastUsedBuilding(rooms)).toBe("별관");
  });
});

describe("buildingNames", () => {
  test("겹치지 않게 가나다순", () => {
    const rooms = [room(1, "본관"), room(2, "별관"), room(3, "본관")];

    expect(buildingNames(rooms)).toEqual(["별관", "본관"]);
  });
});

describe("hasSeveralBuildings", () => {
  test("건물이 하나뿐이면 false", () => {
    expect(hasSeveralBuildings([room(1, "본관"), room(2, "본관")])).toBe(false);
    expect(hasSeveralBuildings([])).toBe(false);
  });

  test("건물이 둘 이상이면 true", () => {
    expect(hasSeveralBuildings([room(1, "본관"), room(2, "별관")])).toBe(true);
  });
});

describe("toRoomRequest", () => {
  test("층을 고르지 않았으면 null로 보낸다", () => {
    expect(
      toRoomRequest({ name: "다목적실", building: "본관", floor: "" }),
    ).toEqual({ name: "다목적실", building: "본관", floor: null });
  });

  test("고른 층은 숫자로 보낸다", () => {
    expect(
      toRoomRequest({ name: "체력단련실", building: "본관", floor: "-1" })
        .floor,
    ).toBe(-1);
  });
});

describe("roomFormErrorsFrom", () => {
  test("입력 오류는 칸마다 나눈다", () => {
    const error = new HostError(
      422,
      "입력을 확인해 주세요",
      "INVALID_INPUT",
      null,
      [
        { field: "name", message: "강의실 이름을 입력해 주세요" },
        { field: "floor", message: "층은 지하 2층부터 5층까지 고릅니다" },
      ],
    );

    expect(roomFormErrorsFrom(error)).toEqual({
      fields: {
        name: "강의실 이름을 입력해 주세요",
        floor: "층은 지하 2층부터 5층까지 고릅니다",
      },
      form: null,
    });
  });

  test("이름이 겹치면 이름 칸에 붙인다 (BR-R1)", () => {
    const error = new HostError(
      409,
      "같은 건물에 같은 이름의 강의실이 있습니다",
      "DUPLICATE_ROOM_NAME",
      "BR-R1",
    );

    expect(roomFormErrorsFrom(error).fields).toEqual({
      name: "같은 건물에 같은 이름의 강의실이 있습니다",
    });
  });

  test("칸에 붙일 수 없는 오류는 폼 전체 문구로", () => {
    const error = new HostError(0, "호스트에 연결하지 못했습니다");

    expect(roomFormErrorsFrom(error)).toEqual({
      fields: {},
      form: "호스트에 연결하지 못했습니다",
    });
  });
});

describe("firstErrorField", () => {
  test("화면에서 가장 위 칸을 고른다", () => {
    expect(firstErrorField({ floor: "a", building: "b" })).toBe("building");
  });
});
