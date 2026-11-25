import type { ChangeKind } from "../../api/generated/ChangeKind";

/**
 * 변경 알림의 종류마다 다시 받을 목록의 키(앞부분).
 *
 * 다른 목록에 함께 보이는 것도 넣는다. 예: 회원 이름은 신청과 선발 결과에도 보인다.
 */
export const CHANGE_KEYS: Record<ChangeKind, readonly string[]> = {
  member: [
    "members",
    "applications",
    "selections",
    "lottery",
    "selection-result",
  ],
  room: ["rooms", "courses"],
  priority_reason: ["priority-reasons", "applications", "lottery"],
  subject: ["subjects", "courses"],
  term: ["terms"],
  course: ["courses", "applicationCounts", "selections", "closed-courses"],
  application: [
    "applications",
    "applicationCounts",
    "selections",
    "lottery",
    "selection-result",
    "closed-courses",
  ],
  selection: [
    "selections",
    "lottery",
    "closed-courses",
    "selection-result",
    "applications",
    "applicationCounts",
  ],
  staff: ["staff", "loginStaff"],
  device: ["devices", "deviceRequests"],
  backup: ["backups"],
};
