import { getJson, patchJson, postJson, sendDelete } from "../../../api/client";
import type { CreateRoomResponse } from "../../../api/generated/CreateRoomResponse";
import type { RoomListResponse } from "../../../api/generated/RoomListResponse";
import type { RoomRequest } from "../../../api/generated/RoomRequest";
import type { RoomResponse } from "../../../api/generated/RoomResponse";

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
