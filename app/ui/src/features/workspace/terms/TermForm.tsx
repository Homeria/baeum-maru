import { useRef, useState, type FormEvent, type RefObject } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { TermResponse } from "../../../api/generated/TermResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { DateField } from "../../../components/ui/DateField";
import { TextField } from "../../../components/ui/TextField";
import { createTerm, deleteTerm, updateTerm } from "./api";
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
import styles from "./TermForm.module.css";

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
  /** 대화상자 안에서 열릴 때 처음 초점을 둘 이름 칸 참조. */
  nameInputRef?: RefObject<HTMLInputElement | null>;
}

/** 학기 입력 칸. 새 학기 등록과 학기 고치기·지우기를 함께 맡는다. */
export function TermForm({
  editing,
  onSaved,
  onDeleted,
  onCancel,
  nameInputRef,
}: TermFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<TermFormValues>(() =>
    editing ? termFormValuesFrom(editing) : EMPTY_TERM_FORM,
  );
  const [errors, setErrors] = useState<TermFormErrors>(NO_FORM_ERRORS);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const cancelDeleteRef = useRef<HTMLButtonElement>(null);
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
    mutationFn: (nextValues: TermFormValues) => {
      const request = toTermRequest(nextValues);
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

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <TextField
        ref={(element) => {
          inputs.current.name = element;
          if (nameInputRef) {
            nameInputRef.current = element;
          }
        }}
        label="이름"
        required
        autoComplete="off"
        placeholder="예: 2027년 1학기"
        value={values.name}
        onChange={(event) => change("name", event.target.value)}
        error={errors.fields.name}
      />

      <DateField
        ref={(element) => {
          inputs.current.startsOn = element;
        }}
        label="시작일"
        required
        placeholder="예: 2027-03-02"
        hint="숫자 8자리 입력 (예: 2027-03-02)"
        value={values.startsOn}
        onChange={(value) => change("startsOn", value)}
        error={errors.fields.startsOn}
      />

      <DateField
        ref={(element) => {
          inputs.current.endsOn = element;
        }}
        label="종료일"
        required
        placeholder="예: 2027-06-30"
        hint="숫자 8자리 입력 (예: 2027-06-30)"
        value={values.endsOn}
        onChange={(value) => change("endsOn", value)}
        error={errors.fields.endsOn}
      />

      {errors.form && <NoticeBanner tone="error" message={errors.form} />}

      <div className={styles.actions}>
        <div className={styles.actionsLeft}>
          {editing && (
            <Button
              variant="danger"
              disabled={busy}
              onClick={() => setConfirmingDelete(true)}
            >
              지우기
            </Button>
          )}
        </div>
        <div className={styles.actionsRight}>
          {onCancel && (
            <Button variant="secondary" disabled={busy} onClick={onCancel}>
              그만두기
            </Button>
          )}
          <Button
            type="submit"
            variant="primary"
            busy={save.isPending}
            busyLabel={editing ? "저장하는 중…" : "등록하는 중…"}
            disabled={remove.isPending}
          >
            {editing ? "저장" : "등록"}
          </Button>
        </div>
      </div>

      <ModalDialog
        open={Boolean(editing && confirmingDelete)}
        size="sm"
        title={`${editing?.name ?? ""}을(를) 지울까요?`}
        busy={remove.isPending}
        initialFocusRef={cancelDeleteRef}
        onClose={() => setConfirmingDelete(false)}
        footer={
          <>
            <Button
              ref={cancelDeleteRef}
              variant="secondary"
              disabled={remove.isPending}
              onClick={() => setConfirmingDelete(false)}
            >
              그만두기
            </Button>
            <Button
              variant="dangerSolid"
              busy={remove.isPending}
              busyLabel="지우는 중…"
              onClick={() => {
                if (editing) {
                  remove.mutate(editing.id);
                }
              }}
            >
              지우기
            </Button>
          </>
        }
      >
        <p className={styles.confirmText}>
          지우면 되돌릴 수 없습니다. 계속할까요?
        </p>
      </ModalDialog>
    </form>
  );
}
