import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import {
  createPriorityReason,
  deletePriorityReason,
  updatePriorityReason,
} from "./api";
import { DeleteDialog } from "../../../components/dialog/DeleteDialog";
import styles from "../../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import {
  EMPTY_PRIORITY_REASON_FORM,
  PRIORITY_METHODS,
  firstErrorField,
  priorityReasonFormErrorsFrom,
  priorityReasonFormValuesFrom,
  toPriorityReasonRequest,
  type PriorityReasonField,
  type PriorityReasonFormErrors,
  type PriorityReasonFormValues,
} from "./priorityReasonFormData";

/** {@link PriorityReasonForm}의 속성. */
export interface PriorityReasonFormProps {
  /** 고칠 사유. 없으면 새 사유 등록. */
  editing?: PriorityReasonResponse;
  /** 처음에 보여 줄 알림. 예: 사유를 지운 뒤 등록 칸으로 돌아왔을 때 */
  initialNotice?: string | null;
  /** 고치기를 끝내고 등록 칸으로 돌아간다. `notice`는 돌아간 칸에 보여 줄 알림. */
  onClose: (notice?: string) => void;
}

/**
 * 우선 배정 사유 입력 칸. 등록과 고치기를 함께 맡는다(BR-S10, BR-S11).
 *
 * 신청에 붙인 적이 있는 사유는 지울 수 없고 이름만 고칠 수 있다. 호스트가 막는다.
 */
export function PriorityReasonForm({
  editing,
  initialNotice,
  onClose,
}: PriorityReasonFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<PriorityReasonFormValues>(() =>
    editing
      ? priorityReasonFormValuesFrom(editing)
      : EMPTY_PRIORITY_REASON_FORM,
  );
  const [errors, setErrors] =
    useState<PriorityReasonFormErrors>(NO_FORM_ERRORS);
  const [notice, setNotice] = useState<string | null>(initialNotice ?? null);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const nameInput = useRef<HTMLInputElement | null>(null);
  const methodInput = useRef<HTMLInputElement | null>(null);

  const showErrors = (error: unknown) => {
    const next = priorityReasonFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field === "name") {
      nameInput.current?.focus();
    } else if (field === "method") {
      methodInput.current?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (values: PriorityReasonFormValues) => {
      const request = toPriorityReasonRequest(values);
      return editing
        ? updatePriorityReason(editing.id, request).then(
            (reason) => reason.name,
          )
        : createPriorityReason(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
      setErrors(NO_FORM_ERRORS);
      if (editing) {
        setNotice(`저장했습니다: ${name}`);
      } else {
        setValues(EMPTY_PRIORITY_REASON_FORM);
        setNotice(`등록했습니다: ${name}`);
        nameInput.current?.focus();
      }
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deletePriorityReason,
    onSuccess: () => {
      setConfirmingDelete(false);
      onClose(`지웠습니다: ${editing?.name ?? ""}`);
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = <F extends PriorityReasonField>(
    field: F,
    value: PriorityReasonFormValues[F],
  ) => {
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

  const nameError = errors.fields.name;
  const methodError = errors.fields.method;

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>
        {editing ? "우선 배정 사유 고치기" : "새 우선 배정 사유"}
      </h2>

      <div className={styles.field}>
        <label htmlFor="priority-reason-name" className={styles.label}>
          이름
          <span className={styles.required}>필수</span>
        </label>
        <input
          id="priority-reason-name"
          ref={nameInput}
          className={styles.input}
          type="text"
          autoComplete="off"
          placeholder="예: 지난 학기 반장"
          aria-invalid={nameError !== undefined}
          aria-describedby={
            nameError ? "priority-reason-name-error" : undefined
          }
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
        />
        {nameError && (
          <p id="priority-reason-name-error" className={styles.fieldError}>
            {nameError}
          </p>
        )}
      </div>

      <fieldset className={styles.field}>
        <legend className={styles.label}>배정 방식</legend>
        <div className={styles.choices}>
          {PRIORITY_METHODS.map((option, index) => (
            <label key={option.value} className={styles.choice}>
              <input
                ref={index === 0 ? methodInput : undefined}
                type="radio"
                name="priority-reason-method"
                value={option.value}
                checked={values.method === option.value}
                onChange={() => change("method", option.value)}
              />
              {option.label}
            </label>
          ))}
        </div>
        <p>
          {
            PRIORITY_METHODS.find((option) => option.value === values.method)
              ?.description
          }
        </p>
        {methodError && <p className={styles.fieldError}>{methodError}</p>}
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
              새 사유 등록으로
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
          title={`${editing.name} 사유를 지울까요?`}
          deleting={remove.isPending}
          onConfirm={() => remove.mutate(editing.id)}
          onCancel={() => setConfirmingDelete(false)}
        />
      )}
    </form>
  );
}
