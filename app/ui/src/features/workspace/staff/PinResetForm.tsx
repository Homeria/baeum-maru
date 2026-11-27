import { useId, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { StaffResponse } from "../../../api/generated/StaffResponse";
import {
  NO_FORM_ERRORS,
  formErrorsFrom,
  type FormErrors,
} from "../../../components/form/formErrors";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import { pinConfirmError } from "../../gate/auth/pinConfirm";
import { resetStaffPin } from "./api";
import styles from "./StaffPage.module.css";

type PinField = "pin" | "pinConfirm";

/** {@link PinResetForm}의 속성. */
export interface PinResetFormProps {
  staff: StaffResponse;
  onSaved: (notice: string) => void;
  onClose: () => void;
}

/**
 * PIN 초기화 대화상자(BR-K10): 직원이 PIN을 잊었거나 잠겼을 때 관리자가 새로 정해 준다.
 *
 * 계정이 잠겼어도 새로 정하면 풀린다. 관리자 자신의 PIN도 여기서 바꾼다.
 */
export function PinResetForm({ staff, onSaved, onClose }: PinResetFormProps) {
  const formId = useId();
  const queryClient = useQueryClient();
  const [pin, setPin] = useState("");
  const [pinConfirm, setPinConfirm] = useState("");
  const [errors, setErrors] = useState<FormErrors<PinField>>(NO_FORM_ERRORS);
  const pinInputRef = useRef<HTMLInputElement>(null);

  const reset = useMutation({
    mutationFn: (nextPin: string) => resetStaffPin(staff.id, nextPin),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
      onSaved(
        `${staff.name}의 PIN을 새로 정했습니다. 잠겨 있었다면 풀렸습니다.`,
      );
    },
    onError: (error) => {
      setErrors(formErrorsFrom(error, ["pin", "pinConfirm"]));
    },
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (reset.isPending) {
      return;
    }
    const mismatch = pinConfirmError(pin, pinConfirm);
    if (mismatch) {
      setErrors({ fields: { pinConfirm: mismatch }, form: null });
      return;
    }
    reset.mutate(pin);
  };

  const minimum = staff.role === "admin" ? "6~8자리" : "4~8자리";

  return (
    <ModalDialog
      open
      size="sm"
      title="PIN 새로 정하기"
      busy={reset.isPending}
      initialFocusRef={pinInputRef}
      onClose={onClose}
      footer={
        <>
          <Button
            variant="secondary"
            disabled={reset.isPending}
            onClick={onClose}
          >
            그만두기
          </Button>
          <Button
            type="submit"
            form={formId}
            variant="primary"
            busy={reset.isPending}
            busyLabel="정하는 중…"
          >
            PIN 새로 정하기
          </Button>
        </>
      }
    >
      <form id={formId} className={styles.form} onSubmit={submit} noValidate>
        <p className={styles.hint}>
          {staff.name}이(가) PIN을 잊었거나 계정이 잠겼을 때 씁니다. 숫자{" "}
          {minimum}.
        </p>

        <TextField
          ref={pinInputRef}
          label="새 PIN"
          required
          type="password"
          inputMode="numeric"
          autoComplete="new-password"
          maxLength={8}
          value={pin}
          onChange={(event) => {
            setPin(event.target.value);
            setErrors(NO_FORM_ERRORS);
          }}
          error={errors.fields.pin}
        />

        <TextField
          label="새 PIN 한 번 더"
          required
          type="password"
          inputMode="numeric"
          autoComplete="new-password"
          maxLength={8}
          value={pinConfirm}
          onChange={(event) => {
            setPinConfirm(event.target.value);
            setErrors(NO_FORM_ERRORS);
          }}
          error={errors.fields.pinConfirm}
        />

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}
      </form>
    </ModalDialog>
  );
}
