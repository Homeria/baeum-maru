import type { SubjectRequest } from "../../api/generated/SubjectRequest";
import type { SubjectResponse } from "../../api/generated/SubjectResponse";
import {
  firstErrorField as firstOf,
  formErrorsFrom,
  type FormErrors,
} from "../../components/form/formErrors";

/** 입력 칸 이름. 호스트 오류의 `field`와 같다. */
export type SubjectField = keyof SubjectRequest;

/** 입력 칸에 들어 있는 글자 그대로. 분류를 비웠으면 `""`. */
export interface SubjectFormValues {
  name: string;
  category: string;
}

export const EMPTY_SUBJECT_FORM: SubjectFormValues = { name: "", category: "" };

/** 화면에 보이는 칸 순서. 오류가 나면 이 순서로 첫 칸에 커서를 둔다. */
export const SUBJECT_FIELDS: readonly SubjectField[] = ["name", "category"];

/** 고칠 과목으로 입력 칸을 채운다. 분류가 없으면 `""`. */
export function subjectFormValuesFrom(
  subject: SubjectResponse,
): SubjectFormValues {
  return { name: subject.name, category: subject.category ?? "" };
}

/** 분류 칸 자동완성에 보여 줄 이름. 겹치지 않게 가나다순. */
export function categoryNames(subjects: readonly SubjectResponse[]): string[] {
  const names = subjects.flatMap((subject) =>
    subject.category === null ? [] : [subject.category],
  );
  return [...new Set(names)].sort((a, b) => a.localeCompare(b, "ko"));
}

/** 분류를 쓴 과목이 있는지. 없으면 목록에서 분류를 숨긴다. */
export function usesCategories(subjects: readonly SubjectResponse[]): boolean {
  return subjects.some((subject) => subject.category !== null);
}

/** 입력 칸 값을 호스트 요청으로 바꾼다. 비운 분류는 `null`. 확인은 호스트가 한다. */
export function toSubjectRequest(values: SubjectFormValues): SubjectRequest {
  return {
    name: values.name,
    category: values.category.trim() === "" ? null : values.category,
  };
}

/** 저장 실패를 칸별 문구와 칸에 붙일 수 없는 문구로 나눈다. */
export type SubjectFormErrors = FormErrors<SubjectField>;

/** 이름이 겹치면(BR-J1) 이름 칸에 붙인다. */
export function subjectFormErrorsFrom(error: unknown): SubjectFormErrors {
  return formErrorsFrom(error, SUBJECT_FIELDS, {
    DUPLICATE_SUBJECT_NAME: "name",
  });
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField(
  fields: Partial<Record<SubjectField, string>>,
): SubjectField | null {
  return firstOf(fields, SUBJECT_FIELDS);
}
