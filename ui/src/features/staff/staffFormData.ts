import type { CreateStaffRequest } from "../../api/generated/CreateStaffRequest";
import type { RoleCode } from "../../api/generated/RoleCode";
import type { StaffResponse } from "../../api/generated/StaffResponse";
import type { UpdateStaffRequest } from "../../api/generated/UpdateStaffRequest";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. `pinConfirm`은 화면에만 있다. */
export type StaffField =
  "name" | "role" | "validUntil" | "pin" | "pinConfirm" | "disabled";

/** 입력 칸에 들어 있는 값 그대로. 날짜는 `YYYY-MM-DD`, 비웠으면 `""`. */
export interface StaffFormValues {
  name: string;
  role: RoleCode;
  validUntil: string;
  /** 새 직원 등록에서만 쓴다. */
  pin: string;
  pinConfirm: string;
  /** 고치기에서만 쓴다. */
  disabled: boolean;
}

export const EMPTY_STAFF_FORM: StaffFormValues = {
  name: "",
  role: "staff",
  validUntil: "",
  pin: "",
  pinConfirm: "",
  disabled: false,
};

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const STAFF_FIELDS: readonly StaffField[] = [
  "name",
  "role",
  "validUntil",
  "pin",
  "pinConfirm",
  "disabled",
];

/** 역할마다 할 수 있는 일. 업무 규칙의 역할 표(BR-K3)를 줄인 것. */
export const ROLE_DESCRIPTIONS: Record<RoleCode, string> = {
  admin: "모든 일과 직원 관리, 접속 기록 보기",
  staff: "회원·강좌·기초 정보를 모두 바꿀 수 있음 (직원 관리는 못 함)",
  temp_staff: "회원 보기와 등록·고치기만 (사회복무요원, 자원봉사자 등)",
  read_only: "보기만",
};

/** 고칠 직원으로 입력 칸을 채운다. PIN 칸은 비운다. */
export function staffFormValuesFrom(staff: StaffResponse): StaffFormValues {
  return {
    ...EMPTY_STAFF_FORM,
    name: staff.name,
    role: staff.role,
    validUntil: staff.validUntil ?? "",
    disabled: staff.disabled,
  };
}

/** 새 직원 요청 본문. 비운 유효기간은 `null`. 확인은 호스트가 한다. */
export function toCreateStaffRequest(
  values: StaffFormValues,
): CreateStaffRequest {
  return {
    name: values.name,
    role: values.role,
    validUntil: values.validUntil === "" ? null : values.validUntil,
    pin: values.pin,
  };
}

/** 고치기 요청 본문. */
export function toUpdateStaffRequest(
  values: StaffFormValues,
): UpdateStaffRequest {
  return {
    name: values.name,
    role: values.role,
    validUntil: values.validUntil === "" ? null : values.validUntil,
    disabled: values.disabled,
  };
}

export type StaffFormErrors = FormErrors<StaffField>;

/**
 * 저장 실패를 칸에 나눠 붙인다. 이름 겹침(BR-K2)은 이름 칸에.
 * 마지막 관리자(BR-K9)는 전체 문구로.
 */
export function staffFormErrorsFrom(error: unknown): StaffFormErrors {
  return formErrorsFrom(error, STAFF_FIELDS, {
    DUPLICATE_STAFF_NAME: "name",
  });
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<StaffField, string>>,
): StaffField | null {
  return firstOf(fields, STAFF_FIELDS);
}

/** 목록의 상태 칸. `today`는 이 PC의 날짜 `YYYY-MM-DD`. */
export function staffStatusLabel(staff: StaffResponse, today: string): string {
  if (staff.disabled) {
    return "사용 중지";
  }
  if (staff.validUntil !== null && staff.validUntil < today) {
    return "기간 끝남";
  }
  if (staff.locked) {
    return "잠김";
  }
  return "사용 중";
}

/** 이 PC의 오늘 날짜 `YYYY-MM-DD`. */
export function todayText(now: Date = new Date()): string {
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
}
