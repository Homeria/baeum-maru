import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { Gender } from "../../../api/generated/Gender";
import type { MemberResponse } from "../../../api/generated/MemberResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Drawer } from "../../../components/overlay/Drawer";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { DateField } from "../../../components/ui/DateField";
import { Radio } from "../../../components/ui/Radio";
import { TextField } from "../../../components/ui/TextField";
import { useCan } from "../../gate/auth/useCan";
import { createMember, deleteMember, updateMember } from "./api";
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
import styles from "./MemberForm.module.css";

const GENDER_OPTIONS: readonly { value: Gender | ""; label: string }[] = [
  { value: "", label: "선택 안 함" },
  { value: "male", label: "남" },
  { value: "female", label: "여" },
];

/** {@link MemberForm}의 속성. */
export interface MemberFormProps {
  /** 고칠 회원. 없으면 새 회원 등록. */
  editing?: MemberResponse;
  /** 저장이나 등록을 끝낸 뒤 서랍을 닫으며 보여 줄 알림을 넘긴다. */
  onSaved: (notice: string) => void;
  /** 회원을 지운 뒤 서랍을 닫으며 보여 줄 알림을 넘긴다. */
  onDeleted?: (notice: string) => void;
  /** 저장하지 않고 서랍을 닫는다. */
  onClose: () => void;
}

/** 회원 등록과 정보 고치기를 맡는 오른쪽 편집 서랍. */
export function MemberForm({
  editing,
  onSaved,
  onDeleted,
  onClose,
}: MemberFormProps) {
  const formId = useId();
  const queryClient = useQueryClient();
  const initialValues = editing
    ? memberFormValuesFrom(editing)
    : EMPTY_MEMBER_FORM;
  const [values, setValues] = useState<MemberFormValues>(() => initialValues);
  const [errors, setErrors] = useState<MemberFormErrors>(NO_FORM_ERRORS);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const canDelete = useCan("deleteMembers");
  const cancelDeleteRef = useRef<HTMLButtonElement>(null);
  const inputs = useRef<Partial<Record<MemberField, HTMLInputElement | null>>>(
    {},
  );

  const isDirty =
    values.memberNo !== initialValues.memberNo ||
    values.name !== initialValues.name ||
    values.phoneNumber !== initialValues.phoneNumber ||
    values.birthDate !== initialValues.birthDate ||
    values.gender !== initialValues.gender;

  const showErrors = (error: unknown) => {
    const next = memberFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (nextValues: MemberFormValues) => {
      const request = toMemberRequest(nextValues);
      return editing
        ? updateMember(editing.id, request).then((member) => member.name)
        : createMember(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["members"] });
      setErrors(NO_FORM_ERRORS);
      onSaved(
        editing ? `${name}님 정보를 저장했습니다` : `${name}님을 등록했습니다`,
      );
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteMember,
    onSuccess: () => {
      setConfirmingDelete(false);
      // 목록만 새로 고침. 지운 회원 하나 보기를 다시 받으면 404가 남
      void queryClient.invalidateQueries({ queryKey: ["members", "search"] });
      (onDeleted ?? onSaved)(`${editing?.name ?? ""}님을 지웠습니다`);
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = (field: MemberField, value: string) => {
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
      title={editing ? "회원 정보 고치기" : "새 회원 등록"}
      subtitle={
        editing
          ? `회원번호 ${editing.memberNo}`
          : "필수 항목을 입력한 뒤 등록하세요"
      }
      isDirty={isDirty}
      busy={busy}
      onClose={onClose}
      cancelLabel="그만두기"
      footerLeft={
        editing && canDelete ? (
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
            inputs.current.memberNo = element;
          }}
          label="회원번호"
          required
          autoComplete="off"
          value={values.memberNo}
          onChange={(event) => change("memberNo", event.target.value)}
          error={errors.fields.memberNo}
        />

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

        <TextField
          ref={(element) => {
            inputs.current.phoneNumber = element;
          }}
          label="연락처"
          required
          inputMode="tel"
          autoComplete="off"
          value={values.phoneNumber}
          onChange={(event) => change("phoneNumber", event.target.value)}
          error={errors.fields.phoneNumber}
        />

        <DateField
          ref={(element) => {
            inputs.current.birthDate = element;
          }}
          label="생년월일"
          placeholder="예: 1955-03-15"
          value={values.birthDate}
          onChange={(value) => change("birthDate", value)}
          error={errors.fields.birthDate}
        />

        <fieldset className={styles.fieldset}>
          <legend className={styles.legend}>성별</legend>
          <div className={styles.choices}>
            {GENDER_OPTIONS.map((option, index) => (
              <Radio
                key={option.value}
                ref={
                  index === 0
                    ? (element) => {
                        inputs.current.gender = element;
                      }
                    : undefined
                }
                name="member-gender"
                value={option.value}
                label={option.label}
                checked={values.gender === option.value}
                onChange={() => change("gender", option.value)}
              />
            ))}
          </div>
          {errors.fields.gender && (
            <p className={styles.fieldError} role="alert">
              {errors.fields.gender}
            </p>
          )}
        </fieldset>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}
      </form>

      <ModalDialog
        open={Boolean(editing && confirmingDelete)}
        size="sm"
        title={`${editing?.name ?? ""}님을 지울까요?`}
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
