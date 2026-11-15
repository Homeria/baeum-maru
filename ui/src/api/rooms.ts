import { getJson, patchJson, postJson, sendDelete } from "./client";
import type { CreateRoomResponse } from "./generated/CreateRoomResponse";
import type { RoomListResponse } from "./generated/RoomListResponse";
import type { RoomRequest } from "./generated/RoomRequest";
import type { RoomResponse } from "./generated/RoomResponse";

/** 강의실을 모두 받는다. 건물, 층, 이름 순서. */
export function listRooms(): Promise<RoomListResponse> {
  return getJson<RoomListResponse>("/api/rooms");
}

/** 새 강의실을 저장하고 새 id를 받는다. */
export function createRoom(request: RoomRequest): Promise<CreateRoomResponse> {
  return postJson<CreateRoomResponse>("/api/rooms", request);
}

/** 강의실의 칸을 모두 바꾸고 고친 뒤 모습을 받는다. */
export function updateRoom(
  id: number,
  request: RoomRequest,
): Promise<RoomResponse> {
  return patchJson<RoomResponse>(`/api/rooms/${id}`, request);
}

/** 강의실을 지운다. */
export function deleteRoom(id: number): Promise<void> {
  return sendDelete(`/api/rooms/${id}`);
}
