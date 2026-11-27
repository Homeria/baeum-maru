import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Drawer } from "../../../components/overlay/Drawer";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { Radio } from "../../../components/ui/Radio";
import { TextField } from "../../../components/ui/TextField";
import {
  createPriorityReason,
  deletePriorityReason,
  updatePriorityReason,
} from "./api";
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
import styles from "./PriorityReasonsPage.module.css";

/** {@link PriorityReasonForm}의 속성. */
export interface PriorityReasonFormProps {
  /** 고칠 사유. 없으면 새 사유 등록. */
  editing?: PriorityReasonResponse;
  /** 저장이나 등록을 끝낸 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onSaved: (notice: string) => void;
  /** 사유를 지운 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onDeleted?: (notice: string) => void;
  /** 저장하지 않고 서랍을 닫는다. */
  onClose: () => void;
}

/**
 * 우선 배정 사유 등록과 고치기를 맡는 오른쪽 편집 서랍(BR-S10, BR-S11).
 *
 * 신청에 붙인 적이 있는 사유는 지울 수 없고 이름만 고칠 수 있다. 호스트가 막는다.
 */
export function PriorityReasonForm({
  editing,
  onSaved,
  onDeleted,
  onClose,
}: PriorityReasonFormProps) {
  const formId = useId();
  const queryClient = useQueryClient();
  const initialValues = editing
    ? priorityReasonFormValuesFrom(editing)
    : EMPTY_PRIORITY_REASON_FORM;
  const [values, setValues] = useState<PriorityReasonFormValues>(
    () => initialValues,
  );
  const [errors, setErrors] =
    useState<PriorityReasonFormErrors>(NO_FORM_ERRORS);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const cancelDeleteRef = useRef<HTMLButtonElement>(null);
  const nameInput = useRef<HTMLInputElement | null>(null);
  const methodInput = useRef<HTMLInputElement | null>(null);

  const isDirty =
    values.name !== initialValues.name ||
    values.method !== initialValues.method;

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
    mutationFn: (nextValues: PriorityReasonFormValues) => {
      const request = toPriorityReasonRequest(nextValues);
      return editing
        ? updatePriorityReason(editing.id, request).then(
            (reason) => reason.name,
          )
        : createPriorityReason(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
      setErrors(NO_FORM_ERRORS);
      onSaved(editing ? `저장했습니다: ${name}` : `등록했습니다: ${name}`);
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deletePriorityReason,
    onSuccess: () => {
      setConfirmingDelete(false);
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
      (onDeleted ?? onSaved)(`지웠습니다: ${editing?.name ?? ""}`);
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
    <Drawer
      open
      title={editing ? "우선 배정 사유 고치기" : "새 우선 배정 사유"}
      subtitle={
        editing ? editing.name : "우선 배정 이름과 방식을 골라 등록해 주세요"
      }
      isDirty={isDirty}
      busy={busy}
      onClose={onClose}
      cancelLabel="그만두기"
      footerLeft={
        editing ? (
          <Button
            variant="danger"
            disabled={busy}
            onClick={() => setConfirmingDelete(true)}
          >
            지우기
          </Button>
        ) : undefined
      }
      footerRight={
        <Button
          type="submit"
          form={formId}
          variant="primary"
          busy={save.isPending}
          busyLabel={editing ? "저장하는 중…" : "등록하는 중…"}
          disabled={remove.isPending}
        >
          {editing ? "저장" : "등록"}
        </Button>
      }
    >
      <form id={formId} className={styles.form} onSubmit={submit} noValidate>
        <TextField
          ref={nameInput}
          label="이름"
          required
          autoComplete="off"
          placeholder="예: 지난 학기 반장"
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
          error={errors.fields.name}
        />

        <fieldset className={styles.fieldset}>
          <legend className={styles.legend}>배정 방식</legend>
          <div className={styles.choices}>
            {PRIORITY_METHODS.map((option, index) => (
              <Radio
                key={option.value}
                ref={index === 0 ? methodInput : undefined}
                name="priority-reason-method"
                value={option.value}
                label={option.label}
                description={option.description}
                checked={values.method === option.value}
                onChange={() => change("method", option.value)}
              />
            ))}
          </div>
          {errors.fields.method && (
            <p className={styles.fieldError} role="alert">
              {errors.fields.method}
            </p>
          )}
        </fieldset>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}
      </form>

      <ModalDialog
        open={Boolean(editing && confirmingDelete)}
        size="sm"
        title={`${editing?.name ?? ""} 사유를 지울까요?`}
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
    </Drawer>
  );
}
