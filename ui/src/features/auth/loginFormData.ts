import type { LoginRequest } from "../../api/generated/LoginRequest";
import {
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";

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
