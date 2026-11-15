import { describe, expect, it } from "vitest";
import type { RoleCode } from "../../api/generated/RoleCode";
import { can, type Action } from "./roles";

const ACTIONS: readonly Action[] = [
  "view",
  "editMembers",
  "deleteMembers",
  "editCourses",
  "editReferenceData",
  "manageStaff",
  "viewAccessLog",
];

/** 역할마다 할 수 있는 일. 업무 규칙의 역할별 표와 같다. */
const ALLOWED: Record<RoleCode, readonly Action[]> = {
  admin: ACTIONS,
  staff: [
    "view",
    "editMembers",
    "deleteMembers",
    "editCourses",
    "editReferenceData",
  ],
  temp_staff: ["view", "editMembers"],
  read_only: ["view"],
};

describe("can", () => {
  it.each(Object.entries(ALLOWED) as [RoleCode, readonly Action[]][])(
    "%s 역할은 역할별 표대로 할 수 있다",
    (role, allowed) => {
      const actual = ACTIONS.filter((action) => can(role, action));

      expect(actual).toEqual(allowed);
    },
  );
});
