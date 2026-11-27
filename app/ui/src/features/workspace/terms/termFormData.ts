import type { TermRequest } from "../../../api/generated/TermRequest";
import type { TermResponse } from "../../../api/generated/TermResponse";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../../components/form/formErrors";

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. */
export type TermField = keyof TermRequest;

/** 입력 칸에 들어 있는 값 그대로. 날짜는 `YYYY-MM-DD`, 비웠으면 `""`. */
export interface TermFormValues {
  name: string;
  startsOn: string;
  endsOn: string;
}

export const EMPTY_TERM_FORM: TermFormValues = {
  name: "",
  startsOn: "",
  endsOn: "",
};

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const TERM_FIELDS: readonly TermField[] = ["name", "startsOn", "endsOn"];

/** 고칠 학기로 입력 칸을 채운다. */
export function termFormValuesFrom(term: TermResponse): TermFormValues {
  return { name: term.name, startsOn: term.startsOn, endsOn: term.endsOn };
}

/** 입력 칸 값을 요청 본문으로. 비운 날짜는 `null`. 확인은 호스트가 한다. */
export function toTermRequest(values: TermFormValues): TermRequest {
  return {
    name: values.name,
    startsOn: values.startsOn === "" ? null : values.startsOn,
    endsOn: values.endsOn === "" ? null : values.endsOn,
  };
}

/**
 * 지금 보여 줄 학기.
 *
 * 고른 학기가 목록에 있으면 그 학기, 없으면(처음 열었거나 지웠을 때) 시작일이 가장 늦은 학기.
 * 목록은 호스트가 시작일이 늦은 순서로 준다.
 */
export function currentTerm(
  terms: readonly TermResponse[],
  selectedId: number | null,
): TermResponse | null {
  return terms.find((term) => term.id === selectedId) ?? terms[0] ?? null;
}

/** 학기 기간 글자. 예: `2027-03-02 ~ 2027-06-30` */
export function termPeriodLabel(term: TermResponse): string {
  return `${term.startsOn} ~ ${term.endsOn}`;
}

export type TermFormErrors = FormErrors<TermField>;

/** 저장·지우기 실패를 칸에 나눠 붙인다. 이름 겹침(BR-T1)은 이름 칸에. */
export function termFormErrorsFrom(error: unknown): TermFormErrors {
  return formErrorsFrom(error, TERM_FIELDS, { DUPLICATE_TERM_NAME: "name" });
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<TermField, string>>,
): TermField | null {
  return firstOf(fields, TERM_FIELDS);
}
