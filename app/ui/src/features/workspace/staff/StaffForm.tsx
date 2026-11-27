import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { RoleCode } from "../../../api/generated/RoleCode";
import type { StaffResponse } from "../../../api/generated/StaffResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Drawer } from "../../../components/overlay/Drawer";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { Checkbox } from "../../../components/ui/Checkbox";
import { SelectField } from "../../../components/ui/SelectField";
import { TextField } from "../../../components/ui/TextField";
import { pinConfirmError } from "../../gate/auth/pinConfirm";
import { ROLE_LABELS } from "../../gate/auth/roles";
import { createStaff, updateStaff } from "./api";
import {
  EMPTY_STAFF_FORM,
  ROLE_DESCRIPTIONS,
  firstErrorField,
  staffFormErrorsFrom,
  staffFormValuesFrom,
  toCreateStaffRequest,
  toUpdateStaffRequest,
  type StaffField,
  type StaffFormErrors,
  type StaffFormValues,
} from "./staffFormData";
import styles from "./StaffPage.module.css";

const ROLES: readonly RoleCode[] = [
  "admin",
  "staff",
  "temp_staff",
  "read_only",
];

/** {@link StaffForm}의 속성. */
export interface StaffFormProps {
  /** 고칠 직원. 없으면 새 직원 등록. */
  editing?: StaffResponse;
  /** 저장이나 등록을 끝낸 뒤 서랍을 닫으며 알림 문구를 넘긴다. */
  onSaved: (notice: string) => void;
  /** 저장하지 않고 서랍을 닫는다. */
  onClose: () => void;
  /** 고치는 중인 직원의 PIN 새로 정하기 창을 연다. */
  onRequestPinReset?: () => void;
}

/**
 * 직원 등록과 고치기를 맡는 오른쪽 편집 서랍.
 *
 * 등록할 때만 PIN을 넣는다. 고칠 때는 사용 중지를 정하고, 직원은 지우지 않는다(BR-K6).
 */
export function StaffForm({
  editing,
  onSaved,
  onClose,
  onRequestPinReset,
}: StaffFormProps) {
  const formId = useId();
  const queryClient = useQueryClient();
  const initialValues = editing
    ? staffFormValuesFrom(editing)
    : EMPTY_STAFF_FORM;
  const [values, setValues] = useState<StaffFormValues>(() => initialValues);
  const [errors, setErrors] = useState<StaffFormErrors>(NO_FORM_ERRORS);
  const inputs = useRef<
    Partial<Record<StaffField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const isDirty =
    values.name !== initialValues.name ||
    values.role !== initialValues.role ||
    values.validUntil !== initialValues.validUntil ||
    values.disabled !== initialValues.disabled ||
    values.pin !== initialValues.pin ||
    values.pinConfirm !== initialValues.pinConfirm;

  const showErrors = (next: StaffFormErrors) => {
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (nextValues: StaffFormValues) =>
      editing
        ? updateStaff(editing.id, toUpdateStaffRequest(nextValues)).then(
            (staff) => staff.name,
          )
        : createStaff(toCreateStaffRequest(nextValues)).then(() =>
            nextValues.name.trim(),
          ),
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
      void queryClient.invalidateQueries({ queryKey: ["loginStaff"] });
      setErrors(NO_FORM_ERRORS);
      onSaved(editing ? `저장했습니다: ${name}` : `등록했습니다: ${name}`);
    },
    onError: (error) => showErrors(staffFormErrorsFrom(error)),
  });

  const change = <F extends StaffField>(
    field: F,
    value: StaffFormValues[F],
  ) => {
    setValues((current) => ({ ...current, [field]: value }));
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (save.isPending) {
      return;
    }
    if (!editing) {
      const mismatch = pinConfirmError(values.pin, values.pinConfirm);
      if (mismatch) {
        showErrors({ fields: { pinConfirm: mismatch }, form: null });
        return;
      }
    }
    save.mutate(values);
  };

  return (
    <Drawer
      open
      title={editing ? "직원 고치기" : "새 직원 등록"}
      subtitle={
        editing
          ? `${editing.name} · ${ROLE_LABELS[editing.role]}`
          : "로그인 이름, 역할, 처음 PIN을 입력해 주세요"
      }
      isDirty={isDirty}
      busy={save.isPending}
      onClose={onClose}
      cancelLabel="그만두기"
      footerLeft={
        editing && onRequestPinReset ? (
          <Button
            variant="secondary"
            disabled={save.isPending}
            onClick={onRequestPinReset}
          >
            PIN 새로 정하기
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
          hint="로그인할 때 이 이름을 고릅니다."
          autoComplete="off"
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
          error={errors.fields.name}
        />

        <SelectField
          ref={(element) => {
            inputs.current.role = element;
          }}
          label="역할"
          required
          hint={ROLE_DESCRIPTIONS[values.role]}
          value={values.role}
          onChange={(event) => change("role", event.target.value as RoleCode)}
          error={errors.fields.role}
        >
          {ROLES.map((role) => (
            <option key={role} value={role}>
              {ROLE_LABELS[role]}
            </option>
          ))}
        </SelectField>

        <TextField
          ref={(element) => {
            inputs.current.validUntil = element;
          }}
          label="유효기간"
          type="date"
          required={values.role === "temp_staff"}
          hint="이날까지 로그인할 수 있습니다. 비우면 기한 없음."
          value={values.validUntil}
          onChange={(event) => change("validUntil", event.target.value)}
          error={errors.fields.validUntil}
        />

        {editing ? (
          <Checkbox
            label="사용 중지"
            description="켜 두면 이 직원은 로그인할 수 없습니다. 기록 보존을 위해 직원은 지우지 않고 중지합니다."
            checked={values.disabled}
            onChange={(event) => change("disabled", event.target.checked)}
          />
        ) : (
          <>
            <TextField
              ref={(element) => {
                inputs.current.pin = element;
              }}
              label="PIN"
              required
              hint="숫자 4~8자리. 관리자는 6자리 이상."
              type="password"
              inputMode="numeric"
              autoComplete="new-password"
              maxLength={8}
              value={values.pin}
              onChange={(event) => change("pin", event.target.value)}
              error={errors.fields.pin}
            />

            <TextField
              ref={(element) => {
                inputs.current.pinConfirm = element;
              }}
              label="PIN 한 번 더"
              required
              type="password"
              inputMode="numeric"
              autoComplete="new-password"
              maxLength={8}
              value={values.pinConfirm}
              onChange={(event) => change("pinConfirm", event.target.value)}
              error={errors.fields.pinConfirm}
            />
          </>
        )}

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}
      </form>
    </Drawer>
  );
}
