import type { RecoveryRequest } from "../../../api/generated/RecoveryRequest";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../../components/form/formErrors";
import { pinConfirmError } from "./pinConfirm";

/** 입력 칸 이름. `pinConfirm`은 화면에만 있다. */
export type RecoveryField = keyof RecoveryRequest | "pinConfirm";

/** 입력 칸에 들어 있는 값 그대로. 이름을 고르지 않았으면 `staffId`는 `null`. */
export interface RecoveryFormValues {
  staffId: number | null;
  code: string;
  pin: string;
  pinConfirm: string;
}

export const EMPTY_RECOVERY_FORM: RecoveryFormValues = {
  staffId: null,
  code: "",
  pin: "",
  pinConfirm: "",
};

/** 화면에 보이는 칸 순서. */
export const RECOVERY_FIELDS: readonly RecoveryField[] = [
  "staffId",
  "code",
  "pin",
  "pinConfirm",
];

export type RecoveryFormErrors = FormErrors<RecoveryField>;

/**
 * 입력 칸 값을 요청 본문으로.
 *
 * 화면에서는 이름을 골랐는지와 두 번 넣은 PIN이 같은지만 본다.
 * 복구 코드와 PIN 모양은 호스트가 확인한다.
 */
export function toRecoveryRequest(
  values: RecoveryFormValues,
): RecoveryRequest | RecoveryFormErrors {
  if (values.staffId === null) {
    return { fields: { staffId: "관리자 이름을 골라 주세요" }, form: null };
  }
  const mismatch = pinConfirmError(values.pin, values.pinConfirm);
  if (mismatch !== null) {
    return { fields: { pinConfirm: mismatch }, form: null };
  }
  return { staffId: values.staffId, code: values.code, pin: values.pin };
}

/**
 * 실패를 칸에 나눠 붙인다.
 *
 * 틀린 복구 코드는 코드 칸에, 관리자가 아닌 직원을 골랐으면 이름 칸에 붙인다.
 */
export function recoveryFormErrorsFrom(error: unknown): RecoveryFormErrors {
  return formErrorsFrom(error, RECOVERY_FIELDS, {
    WRONG_RECOVERY_CODE: "code",
    RECOVERY_ADMIN_ONLY: "staffId",
  });
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<RecoveryField, string>>,
): RecoveryField | null {
  return firstOf(fields, RECOVERY_FIELDS);
}
