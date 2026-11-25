import type { PriorityMethodCode } from "../../api/generated/PriorityMethodCode";
import type { PriorityReasonRequest } from "../../api/generated/PriorityReasonRequest";
import type { PriorityReasonResponse } from "../../api/generated/PriorityReasonResponse";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. */
export type PriorityReasonField = keyof PriorityReasonRequest;

/** 입력 칸에 들어 있는 값 그대로. */
export interface PriorityReasonFormValues {
  name: string;
  method: PriorityMethodCode;
}

export const EMPTY_PRIORITY_REASON_FORM: PriorityReasonFormValues = {
  name: "",
  method: "guaranteed",
};

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const PRIORITY_REASON_FIELDS: readonly PriorityReasonField[] = [
  "name",
  "method",
];

/** 배정 방식의 화면 이름과 설명. 용어집의 이름과 같다(BR-S11). */
export const PRIORITY_METHODS: readonly {
  value: PriorityMethodCode;
  label: string;
  description: string;
}[] = [
  {
    value: "guaranteed",
    label: "자리 보장",
    description: "추첨 없이 당첨되고, 남은 자리만 추첨합니다",
  },
];

/** 배정 방식의 화면 이름. */
export function priorityMethodLabel(method: PriorityMethodCode): string {
  return (
    PRIORITY_METHODS.find((option) => option.value === method)?.label ?? method
  );
}

/** 고칠 사유로 입력 칸을 채운다. */
export function priorityReasonFormValuesFrom(
  reason: PriorityReasonResponse,
): PriorityReasonFormValues {
  return { name: reason.name, method: reason.method };
}

/** 입력 칸 값을 호스트 요청으로 바꾼다. 확인은 호스트가 한다. */
export function toPriorityReasonRequest(
  values: PriorityReasonFormValues,
): PriorityReasonRequest {
  return { name: values.name, method: values.method };
}

/** 저장 실패를 칸별 문구와 칸에 붙일 수 없는 문구로 나눈다. */
export type PriorityReasonFormErrors = FormErrors<PriorityReasonField>;

/** 이름이 겹치면(BR-S10) 이름 칸에 붙인다. 지울 수 없는 사유는 칸이 아닌 문구로. */
export function priorityReasonFormErrorsFrom(
  error: unknown,
): PriorityReasonFormErrors {
  return formErrorsFrom(error, PRIORITY_REASON_FIELDS, {
    DUPLICATE_PRIORITY_REASON_NAME: "name",
  });
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<PriorityReasonField, string>>,
): PriorityReasonField | null {
  return firstOf(fields, PRIORITY_REASON_FIELDS);
}
