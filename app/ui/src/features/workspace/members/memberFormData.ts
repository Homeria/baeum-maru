import type { Gender } from "../../../api/generated/Gender";
import type { MemberRequest } from "../../../api/generated/MemberRequest";
import type { MemberResponse } from "../../../api/generated/MemberResponse";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../../components/form/formErrors";
import { formatPhoneNumber } from "./formatPhoneNumber";

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. */
export type MemberField = keyof MemberRequest;

/** 입력 칸에 들어 있는 글자 그대로. 빈 선택 칸은 `""`. */
export interface MemberFormValues {
  memberNo: string;
  name: string;
  phoneNumber: string;
  birthDate: string;
  gender: Gender | "";
}

export const EMPTY_MEMBER_FORM: MemberFormValues = {
  memberNo: "",
  name: "",
  phoneNumber: "",
  birthDate: "",
  gender: "",
};

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const MEMBER_FIELDS: readonly MemberField[] = [
  "memberNo",
  "name",
  "phoneNumber",
  "birthDate",
  "gender",
];

/** 고칠 회원으로 입력 칸을 채운다. 연락처는 읽기 쉽게 하이픈을 넣는다. */
export function memberFormValuesFrom(member: MemberResponse): MemberFormValues {
  return {
    memberNo: member.memberNo,
    name: member.name,
    phoneNumber: formatPhoneNumber(member.phoneNumber),
    birthDate: member.birthDate ?? "",
    gender: member.gender ?? "",
  };
}

/** 입력 칸 값을 호스트 요청으로 바꾼다. 비운 선택 칸은 `null`. 확인은 호스트가 한다. */
export function toMemberRequest(values: MemberFormValues): MemberRequest {
  return {
    memberNo: values.memberNo,
    name: values.name,
    phoneNumber: values.phoneNumber,
    birthDate: values.birthDate === "" ? null : values.birthDate,
    gender: values.gender === "" ? null : values.gender,
  };
}

/** 저장 실패를 칸별 문구와 칸에 붙일 수 없는 문구로 나눈다. */
export type MemberFormErrors = FormErrors<MemberField>;

/** 회원번호가 겹치면(BR-M2) 회원번호 칸에 붙인다. */
export function memberFormErrorsFrom(error: unknown): MemberFormErrors {
  return formErrorsFrom(error, MEMBER_FIELDS, {
    DUPLICATE_MEMBER_NO: "memberNo",
  });
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<MemberField, string>>,
): MemberField | null {
  return firstOf(fields, MEMBER_FIELDS);
}
