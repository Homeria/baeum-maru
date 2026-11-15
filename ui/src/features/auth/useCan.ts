import { can, type Action } from "./roles";
import { useSession } from "./useSession";

/** 로그인한 직원이 `action`을 할 수 있는지 (BR-K3). 세션이 없으면 `false`. */
export function useCan(action: Action): boolean {
  const session = useSession();
  return session !== null && can(session.staff.role, action);
}
