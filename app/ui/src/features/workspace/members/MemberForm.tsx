import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createMember, deleteMember, updateMember } from "./api";
import { DeleteDialog } from "../../../components/dialog/DeleteDialog";
import { useCan } from "../../gate/auth/useCan";
import type { Gender } from "../../../api/generated/Gender";
import type { MemberResponse } from "../../../api/generated/MemberResponse";
import {
  EMPTY_MEMBER_FORM,
  firstErrorField,
  memberFormErrorsFrom,
  memberFormValuesFrom,
  toMemberRequest,
  type MemberField,
  type MemberFormErrors,
  type MemberFormValues,
} from "./memberFormData";
import styles from "../../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";

const GENDER_OPTIONS: readonly { value: Gender | ""; label: string }[] = [
  { value: "", label: "선택 안 함" },
  { value: "male", label: "남" },
  { value: "female", label: "여" },
];

/** {@link MemberForm}의 속성. */
export interface MemberFormProps {
  /** 고칠 회원. 없으면 새 회원 등록. */
  editing?: MemberResponse;
  /** 처음에 보여 줄 알림. 예: 회원을 지운 뒤 등록 칸으로 돌아왔을 때 */
  initialNotice?: string | null;
  /** 고치기를 끝내고 등록 칸으로 돌아간다. `notice`는 돌아간 칸에 보여 줄 알림. */
  onClose: (notice?: string) => void;
}

/**
 * 회원 입력 칸. 새 회원 등록과 회원 정보 고치기를 함께 맡는다.
 *
 * 등록하면 칸을 비우고 다음 사람을 받을 준비를 한다.
 */
export function MemberForm({
  editing,
  initialNotice,
  onClose,
}: MemberFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<MemberFormValues>(() =>
    editing ? memberFormValuesFrom(editing) : EMPTY_MEMBER_FORM,
  );
  const [errors, setErrors] = useState<MemberFormErrors>(NO_FORM_ERRORS);
  const [notice, setNotice] = useState<string | null>(initialNotice ?? null);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const canDelete = useCan("deleteMembers");
  const inputs = useRef<Partial<Record<MemberField, HTMLInputElement | null>>>(
    {},
  );

  const showErrors = (error: unknown) => {
    const next = memberFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (values: MemberFormValues) => {
      const request = toMemberRequest(values);
      return editing
        ? updateMember(editing.id, request).then((member) => member.name)
        : createMember(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["members"] });
      setErrors(NO_FORM_ERRORS);
      if (editing) {
        setNotice(`${name}님 정보를 저장했습니다`);
      } else {
        setValues(EMPTY_MEMBER_FORM);
        setNotice(`${name}님을 등록했습니다`);
        inputs.current.memberNo?.focus();
      }
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteMember,
    onSuccess: () => {
      setConfirmingDelete(false);
      onClose(`${editing?.name ?? ""}님을 지웠습니다`);
      // 목록만 새로 고침. 지운 회원 하나 보기를 다시 받으면 404가 남
      void queryClient.invalidateQueries({ queryKey: ["members", "search"] });
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

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
    if (busy) {
      return;
    }
    setNotice(null);
    save.mutate(values);
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
      <h2 className={styles.title}>
        {editing ? "회원 정보 고치기" : "새 회원 등록"}
      </h2>

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

      <div className={styles.actions}>
        <button type="submit" className={styles.submit} disabled={busy}>
          {editing
            ? save.isPending
              ? "저장하는 중…"
              : "저장"
            : save.isPending
              ? "등록하는 중…"
              : "등록"}
        </button>
        {editing && (
          <>
            <button
              type="button"
              className={styles.secondary}
              onClick={() => onClose()}
              disabled={busy}
            >
              새 회원 등록으로
            </button>
            {canDelete && (
              <button
                type="button"
                className={styles.danger}
                onClick={() => setConfirmingDelete(true)}
                disabled={busy}
              >
                지우기
              </button>
            )}
          </>
        )}
      </div>

      {editing && confirmingDelete && (
        <DeleteDialog
          title={`${editing.name}님을 지울까요?`}
          deleting={remove.isPending}
          onConfirm={() => remove.mutate(editing.id)}
          onCancel={() => setConfirmingDelete(false)}
        />
      )}
    </form>
  );
}
