import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { SplitAuthLayout } from "../../../components/shell/SplitAuthLayout";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { SelectField } from "../../../components/ui/SelectField";
import { TextField } from "../../../components/ui/TextField";
import { listLoginStaff } from "./api";
import authStyles from "./auth.module.css";
import { recover } from "./recoveryApi";
import {
  EMPTY_RECOVERY_FORM,
  firstErrorField,
  recoveryFormErrorsFrom,
  toRecoveryRequest,
  type RecoveryField,
  type RecoveryFormErrors,
  type RecoveryFormValues,
} from "./recoveryFormData";

/** {@link RecoveryPage}의 속성. */
export interface RecoveryPageProps {
  /** 로그인 화면으로 돌아간다. */
  onBack: () => void;
  /** PIN을 다시 정했다. `recoveryCode`는 새 복구 코드. */
  onDone: (recoveryCode: string) => void;
}

/**
 * PIN을 잊었을 때 (BR-K10, BR-K12).
 *
 * 직원은 관리자에게 PIN 새로 정하기를 부탁하고, 관리자는 인쇄해 둔 복구 코드로 PIN을 다시 정한다.
 * 다시 정하면 새 복구 코드가 나온다.
 */
export function RecoveryPage({ onBack, onDone }: RecoveryPageProps) {
  const staffList = useQuery({
    queryKey: ["loginStaff"],
    queryFn: listLoginStaff,
  });
  const [values, setValues] = useState<RecoveryFormValues>(EMPTY_RECOVERY_FORM);
  const [errors, setErrors] = useState<RecoveryFormErrors>(NO_FORM_ERRORS);
  const inputs = useRef<
    Partial<Record<RecoveryField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const showErrors = (next: RecoveryFormErrors) => {
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field !== null) {
      inputs.current[field]?.focus();
    }
  };

  const submit = useMutation({
    mutationFn: recover,
    onSuccess: (response) => onDone(response.recoveryCode),
    onError: (error) => showErrors(recoveryFormErrorsFrom(error)),
  });

  const change = <F extends keyof RecoveryFormValues>(
    field: F,
    value: RecoveryFormValues[F],
  ) => {
    setValues((current) => ({ ...current, [field]: value }));
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (submit.isPending) {
      return;
    }
    const request = toRecoveryRequest(values);
    if ("fields" in request) {
      showErrors(request);
      return;
    }
    submit.mutate(request);
  };

  return (
    <SplitAuthLayout
      panelTitle="PIN 복구 안내"
      panelDescription="직원과 임시 직원은 관리자에게 PIN 새로 정하기를 부탁해 주세요. 관리자는 보관해 둔 복구 코드로 새 PIN을 정합니다."
      cardTitle="PIN을 잊었을 때"
      cardSubtitle="복구 코드로 새 PIN을 정하면 새 복구 코드가 발급되고, 쓴 코드는 더 이상 쓸 수 없습니다."
    >
      <form className={authStyles.form} onSubmit={onSubmit} noValidate>
        {staffList.isError ? (
          <NoticeBanner tone="error" message={staffList.error.message} />
        ) : (
          <SelectField
            id="recovery-staffId"
            ref={(element) => {
              inputs.current.staffId = element;
            }}
            label="관리자 이름"
            required
            value={values.staffId ?? ""}
            disabled={staffList.isPending}
            error={errors.fields.staffId}
            onChange={(event) =>
              change(
                "staffId",
                event.target.value === "" ? null : Number(event.target.value),
              )
            }
          >
            <option value="">
              {staffList.isPending ? "불러오는 중…" : "골라 주세요"}
            </option>
            {staffList.data?.items
              .filter((item) => item.role === "admin")
              .map((item) => (
                <option key={item.id} value={item.id}>
                  {item.name}
                </option>
              ))}
          </SelectField>
        )}

        <TextField
          id="recovery-code"
          ref={(element) => {
            inputs.current.code = element;
          }}
          label="복구 코드"
          required
          hint="예: K7QF-M2XA-9TRE-HW4C. 가운데 줄표는 없어도 됩니다."
          type="text"
          autoComplete="off"
          value={values.code}
          error={errors.fields.code}
          onChange={(event) => change("code", event.target.value)}
        />

        <TextField
          id="recovery-pin"
          ref={(element) => {
            inputs.current.pin = element;
          }}
          label="새 PIN (숫자 6~8자리)"
          required
          type="password"
          inputMode="numeric"
          autoComplete="new-password"
          maxLength={8}
          value={values.pin}
          error={errors.fields.pin}
          onChange={(event) => change("pin", event.target.value)}
        />

        <TextField
          id="recovery-pinConfirm"
          ref={(element) => {
            inputs.current.pinConfirm = element;
          }}
          label="새 PIN 한 번 더"
          required
          type="password"
          inputMode="numeric"
          autoComplete="new-password"
          maxLength={8}
          value={values.pinConfirm}
          error={errors.fields.pinConfirm}
          onChange={(event) => change("pinConfirm", event.target.value)}
        />

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}

        <div className={authStyles.actionsBetween}>
          <Button
            variant="secondary"
            onClick={onBack}
            disabled={submit.isPending}
          >
            로그인으로 돌아가기
          </Button>
          <Button
            type="submit"
            variant="primary"
            busy={submit.isPending}
            busyLabel="확인하는 중…"
          >
            PIN 다시 정하기
          </Button>
        </div>
      </form>
    </SplitAuthLayout>
  );
}
