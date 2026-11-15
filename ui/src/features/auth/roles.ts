import type { RoleCode } from "../../api/generated/RoleCode";

/** 역할의 화면 이름. 용어집(docs/domain/GLOSSARY.md)의 이름과 같다. */
export const ROLE_LABELS: Record<RoleCode, string> = {
  admin: "관리자",
  staff: "직원",
  temp_staff: "임시 직원",
  read_only: "조회 전용",
};
