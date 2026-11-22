import type { RoleCode } from "../../api/generated/RoleCode";

/** 역할의 화면 이름. 용어집(docs/domain/GLOSSARY.md)의 이름과 같다. */
export const ROLE_LABELS: Record<RoleCode, string> = {
  admin: "관리자",
  staff: "직원",
  temp_staff: "임시 직원",
  read_only: "조회 전용",
};

/** 화면에서 역할을 늘어놓는 순서. 호스트의 `Role::ALL`과 같다. */
export const ROLE_ORDER: readonly RoleCode[] = [
  "admin",
  "staff",
  "temp_staff",
  "read_only",
];

/** 역할에 따라 할 수 있는 일. 호스트의 `Action`과 같다. */
export type Action =
  | "view"
  | "editMembers"
  | "deleteMembers"
  | "editCourses"
  | "editReferenceData"
  | "takeApplications"
  | "overrideRules"
  | "assignPriority"
  | "runLottery"
  | "managePriorityReasons"
  | "manageStaff"
  | "viewAccessLog";

/**
 * BR-K3: `role`인 직원이 `action`을 할 수 있는지. 호스트의 `Role::can`과 같은 표다.
 *
 * 화면은 할 수 없는 일의 버튼을 숨길 뿐이다. 실제로 막는 것은 호스트다.
 */
export function can(role: RoleCode, action: Action): boolean {
  switch (role) {
    case "admin":
      return true;
    case "staff":
      return (
        action !== "managePriorityReasons" &&
        action !== "manageStaff" &&
        action !== "viewAccessLog"
      );
    case "temp_staff":
      return (
        action === "view" ||
        action === "editMembers" ||
        action === "takeApplications"
      );
    case "read_only":
      return action === "view";
  }
}
