import { HostError } from "../../api/client";
import type { ErrorCode } from "../../api/generated/ErrorCode";

/** 저장 실패를 칸별 문구와 칸에 붙일 수 없는 문구로 나눈 것. `F`는 입력 칸 이름. */
export interface FormErrors<F extends string> {
  fields: Partial<Record<F, string>>;
  form: string | null;
}

/** 오류가 없는 상태. */
export const NO_FORM_ERRORS: FormErrors<never> = { fields: {}, form: null };

/**
 * 호스트 오류를 입력 칸에 나눠 붙인다.
 *
 * - `INVALID_INPUT`은 칸마다. 한 칸에 오류가 둘이면 첫 번째만, 모르는 칸 이름은 버린다.
 * - `fieldFor`에 있는 오류 종류(예: 겹침)는 그 칸에.
 * - 나머지는 칸에 붙이지 않는 전체 문구로.
 *
 * @param fields 화면에 보이는 칸 순서.
 * @param fieldFor 칸 하나에 붙일 오류 종류와 그 칸.
 */
export function formErrorsFrom<F extends string>(
  error: unknown,
  fields: readonly F[],
  fieldFor: Partial<Record<ErrorCode, F>> = {},
): FormErrors<F> {
  if (!(error instanceof HostError)) {
    return { fields: {}, form: "저장하지 못했습니다. 다시 해 주세요" };
  }
  if (error.code === "INVALID_INPUT") {
    const found: Partial<Record<F, string>> = {};
    for (const { field, message } of error.fields) {
      if (isField(fields, field) && found[field] === undefined) {
        found[field] = message;
      }
    }
    return { fields: found, form: null };
  }
  const field = error.code === null ? undefined : fieldFor[error.code];
  if (field !== undefined) {
    const found: Partial<Record<F, string>> = {};
    found[field] = error.message;
    return { fields: found, form: null };
  }
  return { fields: {}, form: error.message };
}

/** 오류가 난 칸 중 화면에서 가장 위에 있는 칸. */
export function firstErrorField<F extends string>(
  errors: Partial<Record<F, string>>,
  fields: readonly F[],
): F | null {
  return fields.find((field) => errors[field] !== undefined) ?? null;
}

function isField<F extends string>(
  fields: readonly F[],
  field: string,
): field is F {
  return (fields as readonly string[]).includes(field);
}
