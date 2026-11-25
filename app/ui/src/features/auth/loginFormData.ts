import type { LoginRequest } from "../../api/generated/LoginRequest";
import type { LoginStaffResponse } from "../../api/generated/LoginStaffResponse";
import type { RoleCode } from "../../api/generated/RoleCode";
import {
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";
import { ROLE_ORDER } from "./roles";

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. */
export type LoginField = keyof LoginRequest;

/** 화면 순서. */
export const LOGIN_FIELDS: readonly LoginField[] = ["staffId", "pin"];

export type LoginFormErrors = FormErrors<LoginField>;

/** 고른 직원과 넣은 PIN을 요청 본문으로. 직원을 고르지 않았으면 오류. */
export function toLoginRequest(
  staffId: number | null,
  pin: string,
): LoginRequest | LoginFormErrors {
  if (staffId === null) {
    return { fields: { staffId: "이름을 골라 주세요" }, form: null };
  }
  return { staffId, pin };
}

/**
 * 로그인 실패를 칸에 나눠 붙인다.
 *
 * PIN이 틀리면 PIN 칸에. 계정 잠김(BR-K7)이나 쓸 수 없는 계정은 전체 문구로.
 */
export function loginFormErrorsFrom(error: unknown): LoginFormErrors {
  return formErrorsFrom(error, LOGIN_FIELDS, { WRONG_PIN: "pin" });
}

/** 로그인 화면에서 한 역할의 직원들. */
export interface LoginStaffGroup {
  role: RoleCode;
  items: LoginStaffResponse[];
}

/**
 * 로그인 화면의 직원을 역할별로 묶는다. 역할은 관리자부터, 묶음 안은 받은 순서(이름 순서) 그대로.
 * 직원이 없는 역할은 뺀다.
 */
export function groupByRole(
  items: readonly LoginStaffResponse[],
): LoginStaffGroup[] {
  return ROLE_ORDER.map((role) => ({
    role,
    items: items.filter((item) => item.role === role),
  })).filter((group) => group.items.length > 0);
}
