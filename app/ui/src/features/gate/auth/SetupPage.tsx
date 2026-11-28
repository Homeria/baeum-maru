import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { HostError } from "../../../api/client";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import {
  SplitAuthLayout,
  type SplitAuthStep,
} from "../../../components/shell/SplitAuthLayout";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import { listLoginStaff, login } from "./api";
import authStyles from "./auth.module.css";
import { setup } from "./setupApi";
import {
  EMPTY_SETUP_FORM,
  checkPinConfirm,
  firstErrorField,
  setupFormErrorsFrom,
  toSetupRequest,
  type SetupField,
  type SetupFormErrors,
  type SetupFormValues,
} from "./setupFormData";

const SETUP_STEPS: readonly SplitAuthStep[] = [
  { label: "관리자 계정 만들기", status: "current" },
  { label: "복구 코드 보관", status: "upcoming" },
  { label: "기초 정보 미리 넣기", status: "upcoming" },
];

/** 처음 설정 1단계를 마친 결과. */
export interface SetupDoneResult {
  /** 한 번만 받는 복구 코드. */
  recoveryCode: string;
  /** 방금 만든 관리자로 자동 로그인까지 성공했는지. */
  autoSignedIn: boolean;
}

/** {@link SetupPage}의 속성. */
export interface SetupPageProps {
  /** 관리자를 만들었다. */
  onDone: (result: SetupDoneResult) => void;
  /** 카드 아래쪽에 둘 보조 영역(예: 실행 모드 바꾸기). */
  footer?: ReactNode;
}

/** 처음 설정: 첫 관리자 계정을 만든다 (BR-K11). 직원이 하나도 없을 때만 뜬다. */
export function SetupPage({ onDone, footer }: SetupPageProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<SetupFormValues>(EMPTY_SETUP_FORM);
  const [errors, setErrors] = useState<SetupFormErrors>(NO_FORM_ERRORS);
  const inputs = useRef<Partial<Record<SetupField, HTMLInputElement | null>>>(
    {},
  );
  // PIN이 React Query mutation variables에 남지 않도록 호출 직후 비우는 ref로 전달한다
  const pendingValuesRef = useRef<SetupFormValues | null>(null);

  const showErrors = (next: SetupFormErrors) => {
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: async (): Promise<SetupDoneResult> => {
      const formValues = pendingValuesRef.current;
      pendingValuesRef.current = null;
      if (formValues === null) {
        throw new Error("제출할 설정 값이 없습니다.");
      }
      const request = toSetupRequest(formValues);
      const { recoveryCode } = await setup(request);
      try {
        const { items } = await listLoginStaff();
        const created =
          items.find((staff) => staff.name === request.name.trim()) ??
          (items.length === 1 ? items[0] : undefined);
        if (!created) {
          return { recoveryCode, autoSignedIn: false };
        }
        await login({ staffId: created.id, pin: request.pin });
        return { recoveryCode, autoSignedIn: true };
      } catch {
        // 복구 코드는 절대 잃어버리지 않고, 자동 로그인만 실패하면 2단계 뒤 로그인 화면으로 넘긴다
        return { recoveryCode, autoSignedIn: false };
      }
    },
    onSuccess: (result) => {
      setValues(EMPTY_SETUP_FORM);
      void queryClient.invalidateQueries({ queryKey: ["setupStatus"] });
      onDone(result);
    },
    onError: (error) => {
      showErrors(setupFormErrorsFrom(error));
      // 다른 PC에서 먼저 설정했으면 다음 화면으로 넘어가게 다시 묻는다
      if (error instanceof HostError && error.code === "SETUP_ALREADY_DONE") {
        void queryClient.invalidateQueries({ queryKey: ["setupStatus"] });
      }
    },
  });

  const change = (field: SetupField, value: string) => {
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
    const mismatch = checkPinConfirm(values);
    if (mismatch) {
      showErrors(mismatch);
      return;
    }
    pendingValuesRef.current = values;
    save.mutate();
  };

  return (
    <SplitAuthLayout
      panelTitle="처음 설정"
      panelDescription="관리자 계정과 복구 코드를 만든 뒤, 원하면 강의실·과목 같은 기초 정보를 미리 넣습니다."
      steps={SETUP_STEPS}
      cardTitle="배움마루 처음 설정"
      cardSubtitle="처음 쓰는 것이라 관리자 계정을 먼저 만듭니다. 관리자는 직원 계정을 만들고 관리합니다."
      cardFooter={footer}
    >
      <form className={authStyles.form} onSubmit={submit} noValidate>
        <TextField
          id="setup-name"
          ref={(element) => {
            inputs.current.name = element;
          }}
          label="관리자 이름"
          required
          hint="로그인할 때 이 이름을 고릅니다."
          type="text"
          autoComplete="off"
          placeholder="예: 홍길동"
          autoFocus
          value={values.name}
          error={errors.fields.name}
          onChange={(event) => change("name", event.target.value)}
        />
        <TextField
          id="setup-pin"
          ref={(element) => {
            inputs.current.pin = element;
          }}
          label="PIN"
          required
          hint="숫자 6~8자리. 관리자 PIN은 6자리 이상입니다."
          type="password"
          inputMode="numeric"
          autoComplete="new-password"
          maxLength={8}
          value={values.pin}
          error={errors.fields.pin}
          onChange={(event) => change("pin", event.target.value)}
        />
        <TextField
          id="setup-pinConfirm"
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
          error={errors.fields.pinConfirm}
          onChange={(event) => change("pinConfirm", event.target.value)}
        />

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}

        <div className={authStyles.actions}>
          <Button
            type="submit"
            variant="primary"
            busy={save.isPending}
            busyLabel="만드는 중…"
          >
            관리자 만들기
          </Button>
        </div>
      </form>
    </SplitAuthLayout>
  );
}
