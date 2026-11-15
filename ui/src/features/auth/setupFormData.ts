import type { SetupRequest } from "../../api/generated/SetupRequest";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";

/** 입력 칸 이름. `pinConfirm`은 화면에만 있다. */
export type SetupField = keyof SetupRequest | "pinConfirm";

/** 입력 칸에 들어 있는 값 그대로. */
export interface SetupFormValues {
  name: string;
  pin: string;
  pinConfirm: string;
}

export const EMPTY_SETUP_FORM: SetupFormValues = {
  name: "",
  pin: "",
  pinConfirm: "",
};

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const SETUP_FIELDS: readonly SetupField[] = [
  "name",
  "pin",
  "pinConfirm",
];

/** 입력 칸 값을 요청 본문으로. 이름과 PIN 확인은 호스트가 한다. */
export function toSetupRequest(values: SetupFormValues): SetupRequest {
  return { name: values.name, pin: values.pin };
}

export type SetupFormErrors = FormErrors<SetupField>;

/**
 * 보내기 전에 화면에서만 확인한다: 두 번 넣은 PIN이 같은지.
 *
 * 다르면 오류, 같으면 `null`. PIN 모양(숫자 6~8자리)은 호스트가 확인한다.
 */
export function checkPinConfirm(
  values: SetupFormValues,
): SetupFormErrors | null {
  if (values.pinConfirm === values.pin) {
    return null;
  }
  return {
    fields: { pinConfirm: "위에 넣은 PIN과 다릅니다" },
    form: null,
  };
}

/** 저장 실패를 칸에 나눠 붙인다. */
export function setupFormErrorsFrom(error: unknown): SetupFormErrors {
  return formErrorsFrom(error, SETUP_FIELDS);
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<SetupField, string>>,
): SetupField | null {
  return firstOf(fields, SETUP_FIELDS);
}
