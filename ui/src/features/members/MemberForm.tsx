import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createMember } from "../../api/members";
import type { Gender } from "../../api/generated/Gender";
import {
  EMPTY_MEMBER_FORM,
  firstErrorField,
  memberFormErrorsFrom,
  toMemberRequest,
  type MemberField,
  type MemberFormErrors,
  type MemberFormValues,
} from "./memberForm";
import styles from "./MemberForm.module.css";

const NO_ERRORS: MemberFormErrors = { fields: {}, form: null };

const GENDER_OPTIONS: readonly { value: Gender | ""; label: string }[] = [
  { value: "", label: "선택 안 함" },
  { value: "male", label: "남" },
  { value: "female", label: "여" },
];

/** 새 회원 입력 칸. 등록하면 칸을 비우고 다음 사람을 받을 준비를 한다. */
export function MemberForm() {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<MemberFormValues>(EMPTY_MEMBER_FORM);
  const [errors, setErrors] = useState<MemberFormErrors>(NO_ERRORS);
  const [notice, setNotice] = useState<string | null>(null);
  const inputs = useRef<Partial<Record<MemberField, HTMLInputElement | null>>>(
    {},
  );

  const create = useMutation({
    mutationFn: createMember,
    onSuccess: (_, request) => {
      void queryClient.invalidateQueries({ queryKey: ["members"] });
      setValues(EMPTY_MEMBER_FORM);
      setErrors(NO_ERRORS);
      setNotice(`${request.name.trim()}님을 등록했습니다`);
      inputs.current.memberNo?.focus();
    },
    onError: (error) => {
      const next = memberFormErrorsFrom(error);
      setErrors(next);
      const field = firstErrorField(next.fields);
      if (field) {
        inputs.current[field]?.focus();
      }
    },
  });

  const change = (field: MemberField, value: string) => {
    setValues((current) => ({ ...current, [field]: value }));
    // 고치기 시작한 칸의 오류는 지움
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
    setNotice(null);
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (create.isPending) {
      return;
    }
    setNotice(null);
    create.mutate(toMemberRequest(values));
  };

  const textField = (
    field: MemberField,
    label: string,
    options: { required?: boolean; type?: string; inputMode?: "tel" } = {},
  ) => {
    const errorId = `member-${field}-error`;
    const error = errors.fields[field];
    return (
      <div className={styles.field}>
        <label htmlFor={`member-${field}`} className={styles.label}>
          {label}
          {options.required && <span className={styles.required}>필수</span>}
        </label>
        <input
          id={`member-${field}`}
          ref={(element) => {
            inputs.current[field] = element;
          }}
          className={styles.input}
          type={options.type ?? "text"}
          inputMode={options.inputMode}
          autoComplete="off"
          value={values[field]}
          onChange={(event) => change(field, event.target.value)}
          aria-invalid={error !== undefined}
          aria-describedby={error ? errorId : undefined}
        />
        {error && (
          <p id={errorId} className={styles.fieldError}>
            {error}
          </p>
        )}
      </div>
    );
  };

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>새 회원 등록</h2>

      {textField("memberNo", "회원번호", { required: true })}
      {textField("name", "이름", { required: true })}
      {textField("phoneNumber", "연락처", { required: true, inputMode: "tel" })}
      {textField("birthDate", "생년월일", { type: "date" })}

      <fieldset className={styles.field}>
        <legend className={styles.label}>성별</legend>
        <div className={styles.choices}>
          {GENDER_OPTIONS.map((option, index) => (
            <label key={option.value} className={styles.choice}>
              <input
                ref={
                  index === 0
                    ? (element) => {
                        inputs.current.gender = element;
                      }
                    : undefined
                }
                type="radio"
                name="member-gender"
                value={option.value}
                checked={values.gender === option.value}
                onChange={() => change("gender", option.value)}
              />
              {option.label}
            </label>
          ))}
        </div>
        {errors.fields.gender && (
          <p className={styles.fieldError}>{errors.fields.gender}</p>
        )}
      </fieldset>

      {errors.form && (
        <p className={styles.formError} role="alert">
          {errors.form}
        </p>
      )}
      {notice && (
        <p className={styles.notice} role="status">
          {notice}
        </p>
      )}

      <button
        type="submit"
        className={styles.submit}
        disabled={create.isPending}
      >
        {create.isPending ? "등록하는 중…" : "등록"}
      </button>
    </form>
  );
}
