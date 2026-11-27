import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { TermResponse } from "../../../api/generated/TermResponse";
import { createTerm, deleteTerm, updateTerm } from "./api";
import { DeleteDialog } from "../../../components/dialog/DeleteDialog";
import styles from "../../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import {
  EMPTY_TERM_FORM,
  firstErrorField,
  termFormErrorsFrom,
  termFormValuesFrom,
  toTermRequest,
  type TermField,
  type TermFormErrors,
  type TermFormValues,
} from "./termFormData";

/** {@link TermForm}의 속성. */
export interface TermFormProps {
  /** 고칠 학기. 없으면 새 학기 등록. */
  editing?: TermResponse;
  /** 저장했다. `id`는 저장한 학기, `notice`는 보여 줄 알림. */
  onSaved: (id: number, notice: string) => void;
  /** 지웠다. 고칠 때만 부른다. */
  onDeleted?: (notice: string) => void;
  /** 그만둔다. 없으면 "그만두기"를 보여 주지 않는다 (학기가 하나도 없을 때). */
  onCancel?: () => void;
}

/** 학기 입력 칸. 새 학기 등록과 학기 고치기·지우기를 함께 맡는다. */
export function TermForm({
  editing,
  onSaved,
  onDeleted,
  onCancel,
}: TermFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<TermFormValues>(() =>
    editing ? termFormValuesFrom(editing) : EMPTY_TERM_FORM,
  );
  const [errors, setErrors] = useState<TermFormErrors>(NO_FORM_ERRORS);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const inputs = useRef<Partial<Record<TermField, HTMLInputElement | null>>>(
    {},
  );

  const showErrors = (error: unknown) => {
    const next = termFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (values: TermFormValues) => {
      const request = toTermRequest(values);
      return editing
        ? updateTerm(editing.id, request)
        : createTerm(request).then(({ id }) => ({
            id,
            name: request.name.trim(),
          }));
    },
    onSuccess: ({ id, name }) => {
      void queryClient.invalidateQueries({ queryKey: ["terms"] });
      onSaved(id, editing ? `저장했습니다: ${name}` : `등록했습니다: ${name}`);
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteTerm,
    onSuccess: () => {
      setConfirmingDelete(false);
      void queryClient.invalidateQueries({ queryKey: ["terms"] });
      onDeleted?.(`지웠습니다: ${editing?.name ?? ""}`);
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = (field: TermField, value: string) => {
    setValues((current) => ({ ...current, [field]: value }));
    // 고치기 시작한 칸의 오류는 지움
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    save.mutate(values);
  };

  /** 칸 이름표와 오류 문구 사이에 입력 부품을 끼운다. */
  const field = (
    name: TermField,
    label: string,
    control: (props: {
      id: string;
      "aria-invalid": boolean;
      "aria-describedby": string | undefined;
    }) => ReactNode,
  ) => {
    const id = `term-${name}`;
    const errorId = `${id}-error`;
    const error = errors.fields[name];
    return (
      <div className={styles.field}>
        <label htmlFor={id} className={styles.label}>
          {label}
          <span className={styles.required}>필수</span>
        </label>
        {control({
          id,
          "aria-invalid": error !== undefined,
          "aria-describedby": error ? errorId : undefined,
        })}
        {error && (
          <p id={errorId} className={styles.fieldError}>
            {error}
          </p>
        )}
      </div>
    );
  };

  const dateInput = (name: "startsOn" | "endsOn") =>
    field(name, name === "startsOn" ? "시작일" : "종료일", (props) => (
      <input
        {...props}
        ref={(element) => {
          inputs.current[name] = element;
        }}
        className={styles.input}
        type="date"
        value={values[name]}
        onChange={(event) => change(name, event.target.value)}
      />
    ));

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>
        {editing ? "학기 고치기" : "새 학기 등록"}
      </h2>

      {field("name", "이름", (props) => (
        <input
          {...props}
          ref={(element) => {
            inputs.current.name = element;
          }}
          className={styles.input}
          type="text"
          autoComplete="off"
          placeholder="예: 2027년 1학기"
          // 창이 뜨면 바로 이름부터 쓰게
          autoFocus
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
        />
      ))}
      {dateInput("startsOn")}
      {dateInput("endsOn")}

      {errors.form && (
        <p className={styles.formError} role="alert">
          {errors.form}
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
        {onCancel && (
          <button
            type="button"
            className={styles.secondary}
            onClick={onCancel}
            disabled={busy}
          >
            그만두기
          </button>
        )}
        {editing && (
          <button
            type="button"
            className={styles.danger}
            onClick={() => setConfirmingDelete(true)}
            disabled={busy}
          >
            지우기
          </button>
        )}
      </div>

      {editing && confirmingDelete && (
        <DeleteDialog
          title={`${editing.name}을(를) 지울까요?`}
          deleting={remove.isPending}
          onConfirm={() => remove.mutate(editing.id)}
          onCancel={() => setConfirmingDelete(false)}
        />
      )}
    </form>
  );
}
