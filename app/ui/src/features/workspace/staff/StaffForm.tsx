import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { RoleCode } from "../../../api/generated/RoleCode";
import type { StaffResponse } from "../../../api/generated/StaffResponse";
import { createStaff, updateStaff } from "./api";
import styles from "../../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import authStyles from "../../gate/auth/auth.module.css";
import { pinConfirmError } from "../../gate/auth/pinConfirm";
import { ROLE_LABELS } from "../../gate/auth/roles";
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
  /** 처음에 보여 줄 알림. */
  initialNotice?: string | null;
  /** 고치기를 끝내고 등록 칸으로 돌아간다. */
  onClose: () => void;
}

/**
 * 직원 입력 칸. 새 직원 등록과 직원 고치기를 함께 맡는다.
 *
 * 등록할 때만 PIN을 넣는다. 고칠 때는 사용 중지를 정하고, PIN은 아래 칸에서 따로 새로 정한다.
 * 직원은 지우지 않고 사용 중지한다 (BR-K6).
 */
export function StaffForm({ editing, initialNotice, onClose }: StaffFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<StaffFormValues>(() =>
    editing ? staffFormValuesFrom(editing) : EMPTY_STAFF_FORM,
  );
  const [errors, setErrors] = useState<StaffFormErrors>(NO_FORM_ERRORS);
  const [notice, setNotice] = useState<string | null>(initialNotice ?? null);
  const inputs = useRef<
    Partial<Record<StaffField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const showErrors = (next: StaffFormErrors) => {
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (values: StaffFormValues) =>
      editing
        ? updateStaff(editing.id, toUpdateStaffRequest(values)).then(
            (staff) => staff.name,
          )
        : createStaff(toCreateStaffRequest(values)).then(() =>
            values.name.trim(),
          ),
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
      void queryClient.invalidateQueries({ queryKey: ["loginStaff"] });
      setErrors(NO_FORM_ERRORS);
      if (editing) {
        setNotice(`저장했습니다: ${name}`);
      } else {
        setValues(EMPTY_STAFF_FORM);
        setNotice(`등록했습니다: ${name}`);
        inputs.current.name?.focus();
      }
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
    setNotice(null);
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (save.isPending) {
      return;
    }
    setNotice(null);
    if (!editing) {
      const mismatch = pinConfirmError(values.pin, values.pinConfirm);
      if (mismatch) {
        showErrors({ fields: { pinConfirm: mismatch }, form: null });
        return;
      }
    }
    save.mutate(values);
  };

  /** 칸 이름표, 입력 부품, 도움말, 오류. */
  const field = (
    name: StaffField,
    label: string,
    required: boolean,
    hint: string | null,
    control: (props: {
      id: string;
      "aria-invalid": boolean;
      "aria-describedby": string | undefined;
    }) => ReactNode,
  ) => {
    const id = `staff-${name}`;
    const hintId = `${id}-hint`;
    const errorId = `${id}-error`;
    const error = errors.fields[name];
    const describedBy = [hint ? hintId : null, error ? errorId : null]
      .filter((value) => value !== null)
      .join(" ");
    return (
      <div className={styles.field}>
        <label htmlFor={id} className={styles.label}>
          {label}
          {required && <span className={styles.required}>필수</span>}
        </label>
        {control({
          id,
          "aria-invalid": error !== undefined,
          "aria-describedby": describedBy || undefined,
        })}
        {hint && (
          <p id={hintId} className={authStyles.hint}>
            {hint}
          </p>
        )}
        {error && (
          <p id={errorId} className={styles.fieldError}>
            {error}
          </p>
        )}
      </div>
    );
  };

  const pinInput =
    (name: "pin" | "pinConfirm") =>
    (props: {
      id: string;
      "aria-invalid": boolean;
      "aria-describedby": string | undefined;
    }) => (
      <input
        {...props}
        ref={(element) => {
          inputs.current[name] = element;
        }}
        className={styles.input}
        type="password"
        inputMode="numeric"
        autoComplete="new-password"
        maxLength={8}
        value={values[name]}
        onChange={(event) => change(name, event.target.value)}
      />
    );

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>
        {editing ? "직원 고치기" : "새 직원 등록"}
      </h2>

      {field(
        "name",
        "이름",
        true,
        "로그인할 때 이 이름을 고릅니다.",
        (props) => (
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
        ),
      )}

      {field("role", "역할", true, ROLE_DESCRIPTIONS[values.role], (props) => (
        <select
          {...props}
          ref={(element) => {
            inputs.current.role = element;
          }}
          className={styles.input}
          value={values.role}
          onChange={(event) => change("role", event.target.value as RoleCode)}
        >
          {ROLES.map((role) => (
            <option key={role} value={role}>
              {ROLE_LABELS[role]}
            </option>
          ))}
        </select>
      ))}

      {field(
        "validUntil",
        "유효기간",
        values.role === "temp_staff",
        "이날까지 로그인할 수 있습니다. 비우면 기한 없음.",
        (props) => (
          <input
            {...props}
            ref={(element) => {
              inputs.current.validUntil = element;
            }}
            className={styles.input}
            type="date"
            value={values.validUntil}
            onChange={(event) => change("validUntil", event.target.value)}
          />
        ),
      )}

      {editing ? (
        <label className={styles.choice}>
          <input
            type="checkbox"
            checked={values.disabled}
            onChange={(event) => change("disabled", event.target.checked)}
          />
          사용 중지 (로그인할 수 없음)
        </label>
      ) : (
        <>
          {field(
            "pin",
            "PIN",
            true,
            "숫자 4~8자리. 관리자는 6자리 이상.",
            pinInput("pin"),
          )}
          {field(
            "pinConfirm",
            "PIN 한 번 더",
            true,
            null,
            pinInput("pinConfirm"),
          )}
        </>
      )}

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
        <button
          type="submit"
          className={styles.submit}
          disabled={save.isPending}
        >
          {editing
            ? save.isPending
              ? "저장하는 중…"
              : "저장"
            : save.isPending
              ? "등록하는 중…"
              : "등록"}
        </button>
        {editing && (
          <button
            type="button"
            className={styles.secondary}
            onClick={onClose}
            disabled={save.isPending}
          >
            새 직원 등록으로
          </button>
        )}
      </div>
    </form>
  );
}
