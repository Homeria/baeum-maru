import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { SubjectResponse } from "../../api/generated/SubjectResponse";
import {
  createSubject,
  deleteSubject,
  updateSubject,
} from "../../api/subjects";
import { DeleteDialog } from "../../components/dialog/DeleteDialog";
import styles from "../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../components/form/formErrors";
import {
  EMPTY_SUBJECT_FORM,
  categoryNames,
  firstErrorField,
  subjectFormErrorsFrom,
  subjectFormValuesFrom,
  toSubjectRequest,
  type SubjectField,
  type SubjectFormErrors,
  type SubjectFormValues,
} from "./subjectFormData";

const CATEGORY_LIST_ID = "subject-category-names";

/** {@link SubjectForm}의 속성. */
export interface SubjectFormProps {
  /** 지금 있는 과목. 분류 칸 자동완성에 쓴다. */
  subjects: readonly SubjectResponse[];
  /** 고칠 과목. 없으면 새 과목 등록. */
  editing?: SubjectResponse;
  /** 처음에 보여 줄 알림. 예: 과목을 지운 뒤 등록 칸으로 돌아왔을 때 */
  initialNotice?: string | null;
  /** 고치기를 끝내고 등록 칸으로 돌아간다. `notice`는 돌아간 칸에 보여 줄 알림. */
  onClose: (notice?: string) => void;
}

/**
 * 과목 입력 칸. 새 과목 등록과 과목 고치기를 함께 맡는다.
 *
 * 등록하면 이름만 비운다. 같은 분류의 과목을 이어서 넣기 쉽게 분류는 남긴다.
 */
export function SubjectForm({
  subjects,
  editing,
  initialNotice,
  onClose,
}: SubjectFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<SubjectFormValues>(() =>
    editing ? subjectFormValuesFrom(editing) : EMPTY_SUBJECT_FORM,
  );
  const [errors, setErrors] = useState<SubjectFormErrors>(NO_FORM_ERRORS);
  const [notice, setNotice] = useState<string | null>(initialNotice ?? null);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const inputs = useRef<Partial<Record<SubjectField, HTMLInputElement | null>>>(
    {},
  );

  const showErrors = (error: unknown) => {
    const next = subjectFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (values: SubjectFormValues) => {
      const request = toSubjectRequest(values);
      return editing
        ? updateSubject(editing.id, request).then((subject) => subject.name)
        : createSubject(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
      setErrors(NO_FORM_ERRORS);
      if (editing) {
        setNotice(`저장했습니다: ${name}`);
      } else {
        setValues((current) => ({ ...current, name: "" }));
        setNotice(`등록했습니다: ${name}`);
        inputs.current.name?.focus();
      }
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteSubject,
    onSuccess: () => {
      setConfirmingDelete(false);
      onClose(`지웠습니다: ${editing?.name ?? ""}`);
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = (field: SubjectField, value: string) => {
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

  /** 칸 이름표와 오류 문구 사이에 입력 부품을 끼운다. */
  const field = (
    name: SubjectField,
    label: string,
    required: boolean,
    control: (props: {
      id: string;
      "aria-invalid": boolean;
      "aria-describedby": string | undefined;
    }) => ReactNode,
  ) => {
    const id = `subject-${name}`;
    const errorId = `${id}-error`;
    const error = errors.fields[name];
    return (
      <div className={styles.field}>
        <label htmlFor={id} className={styles.label}>
          {label}
          {required && <span className={styles.required}>필수</span>}
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

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>
        {editing ? "과목 고치기" : "새 과목 등록"}
      </h2>

      {field("name", "이름", true, (props) => (
        <input
          {...props}
          ref={(element) => {
            inputs.current.name = element;
          }}
          className={styles.input}
          type="text"
          autoComplete="off"
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
        />
      ))}

      {field("category", "분류", false, (props) => (
        <>
          <input
            {...props}
            ref={(element) => {
              inputs.current.category = element;
            }}
            className={styles.input}
            type="text"
            autoComplete="off"
            list={CATEGORY_LIST_ID}
            value={values.category}
            onChange={(event) => change("category", event.target.value)}
          />
          <datalist id={CATEGORY_LIST_ID}>
            {categoryNames(subjects).map((category) => (
              <option key={category} value={category} />
            ))}
          </datalist>
        </>
      ))}

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
              새 과목 등록으로
            </button>
            <button
              type="button"
              className={styles.danger}
              onClick={() => setConfirmingDelete(true)}
              disabled={busy}
            >
              지우기
            </button>
          </>
        )}
      </div>

      {editing && confirmingDelete && (
        <DeleteDialog
          title={`${editing.name} 과목을 지울까요?`}
          deleting={remove.isPending}
          onConfirm={() => remove.mutate(editing.id)}
          onCancel={() => setConfirmingDelete(false)}
        />
      )}
    </form>
  );
}
