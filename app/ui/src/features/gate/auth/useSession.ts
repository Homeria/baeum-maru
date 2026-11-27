import { useSyncExternalStore } from "react";
import {
  getSession,
  subscribeSession,
  type Session,
} from "../../../api/session";

/** 지금 세션. 로그인·로그아웃하거나 호스트가 세션을 끝내면 다시 그린다. */
export function useSession(): Session | null {
  return useSyncExternalStore(subscribeSession, getSession);
}
