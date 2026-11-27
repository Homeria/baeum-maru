import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Drawer } from "../../../components/overlay/Drawer";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import { createSubject, deleteSubject, updateSubject } from "./api";
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
import styles from "./SubjectsPage.module.css";

const CATEGORY_LIST_ID = "subject-category-names";

/** {@link SubjectForm}의 속성. */
export interface SubjectFormProps {
  /** 지금 있는 과목. 분류 칸 자동완성에 쓴다. */
  subjects: readonly SubjectResponse[];
  /** 고칠 과목. 없으면 새 과목 등록. */
  editing?: SubjectResponse;
  /** 저장이나 등록을 끝낸 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onSaved: (notice: string) => void;
  /** 과목을 지운 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onDeleted?: (notice: string) => void;
  /** 저장하지 않고 서랍을 닫는다. */
  onClose: () => void;
}

/** 과목 등록과 고치기를 맡는 오른쪽 편집 서랍. */
export function SubjectForm({
  subjects,
  editing,
  onSaved,
  onDeleted,
  onClose,
}: SubjectFormProps) {
  const formId = useId();
  const queryClient = useQueryClient();
  const initialValues = editing
    ? subjectFormValuesFrom(editing)
    : EMPTY_SUBJECT_FORM;
  const [values, setValues] = useState<SubjectFormValues>(() => initialValues);
  const [errors, setErrors] = useState<SubjectFormErrors>(NO_FORM_ERRORS);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const cancelDeleteRef = useRef<HTMLButtonElement>(null);
  const inputs = useRef<Partial<Record<SubjectField, HTMLInputElement | null>>>(
    {},
  );

  const isDirty =
    values.name !== initialValues.name ||
    values.category !== initialValues.category;

  const showErrors = (error: unknown) => {
    const next = subjectFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (nextValues: SubjectFormValues) => {
      const request = toSubjectRequest(nextValues);
      return editing
        ? updateSubject(editing.id, request).then((subject) => subject.name)
        : createSubject(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
      setErrors(NO_FORM_ERRORS);
      onSaved(editing ? `저장했습니다: ${name}` : `등록했습니다: ${name}`);
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteSubject,
    onSuccess: () => {
      setConfirmingDelete(false);
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
      (onDeleted ?? onSaved)(`지웠습니다: ${editing?.name ?? ""}`);
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = (field: SubjectField, value: string) => {
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
      title={editing ? "과목 고치기" : "새 과목 등록"}
      subtitle={
        editing ? editing.name : "과목 이름과 분류(선택)를 입력해 주세요"
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
          ref={(element) => {
            inputs.current.name = element;
          }}
          label="이름"
          required
          autoComplete="off"
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
          error={errors.fields.name}
        />

        <div>
          <TextField
            ref={(element) => {
              inputs.current.category = element;
            }}
            label="분류"
            autoComplete="off"
            list={CATEGORY_LIST_ID}
            value={values.category}
            onChange={(event) => change("category", event.target.value)}
            error={errors.fields.category}
          />
          <datalist id={CATEGORY_LIST_ID}>
            {categoryNames(subjects).map((category) => (
              <option key={category} value={category} />
            ))}
          </datalist>
        </div>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}
      </form>

      <ModalDialog
        open={Boolean(editing && confirmingDelete)}
        size="sm"
        title={`${editing?.name ?? ""} 과목을 지울까요?`}
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
