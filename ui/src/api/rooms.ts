import { getJson, postJson } from "./client";
import type { CreateRoomResponse } from "./generated/CreateRoomResponse";
import type { RoomListResponse } from "./generated/RoomListResponse";
import type { RoomRequest } from "./generated/RoomRequest";

/** 강의실을 모두 받는다. 건물, 층, 이름 순서. */
export function listRooms(): Promise<RoomListResponse> {
  return getJson<RoomListResponse>("/api/rooms");
}

/** 새 강의실을 저장하고 새 id를 받는다. */
export function createRoom(request: RoomRequest): Promise<CreateRoomResponse> {
  return postJson<CreateRoomResponse>("/api/rooms", request);
}
