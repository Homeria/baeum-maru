import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { HostError } from "../../../api/client";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { SplitAuthLayout } from "../../../components/shell/SplitAuthLayout";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { TextField } from "../../../components/ui/TextField";
import { listLoginStaff, login } from "./api";
import authStyles from "./auth.module.css";
import {
  groupByRole,
  loginFormErrorsFrom,
  toLoginRequest,
  type LoginFormErrors,
} from "./loginFormData";
import { ROLE_LABELS } from "./roles";

/** {@link LoginPage}의 속성. */
export interface LoginPageProps {
  /** "PIN을 잊었어요"를 눌렀다. */
  onForgotPin: () => void;
  /** 카드 아래쪽에 둘 보조 영역(예: 실행 모드 바꾸기). */
  footer?: ReactNode;
}

/**
 * 로그인: 이름을 고르고 PIN을 넣는다 (BR-K1, BR-K2).
 *
 * 이름은 역할별로 묶어 보여 준다. 임시 직원이 많아도 찾기 쉽게.
 *
 * 성공하면 세션이 생겨 앱이 메인 화면으로 바뀐다.
 */
export function LoginPage({ onForgotPin, footer }: LoginPageProps) {
  const queryClient = useQueryClient();
  const staffList = useQuery({
    queryKey: ["loginStaff"],
    queryFn: listLoginStaff,
  });
  const [staffId, setStaffId] = useState<number | null>(null);
  const [pin, setPin] = useState("");
  const [errors, setErrors] = useState<LoginFormErrors>(NO_FORM_ERRORS);
  const pinInput = useRef<HTMLInputElement>(null);

  const submit = useMutation({
    mutationFn: login,
    onError: (error) => {
      setErrors(loginFormErrorsFrom(error));
      // 틀린 PIN은 지우고 다시 넣게
      setPin("");
      pinInput.current?.focus();
      // 그사이 사용 중지되었거나 없어진 직원은 목록에서 빠지게
      if (
        error instanceof HostError &&
        (error.code === "STAFF_NOT_FOUND" ||
          error.code === "ACCOUNT_UNAVAILABLE")
      ) {
        void queryClient.invalidateQueries({ queryKey: ["loginStaff"] });
      }
    },
  });

  const choose = (id: number) => {
    setStaffId(id);
    setErrors(NO_FORM_ERRORS);
    pinInput.current?.focus();
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (submit.isPending) {
      return;
    }
    const request = toLoginRequest(staffId, pin);
    if ("fields" in request) {
      setErrors(request);
      return;
    }
    submit.mutate(request);
  };

  const items = staffList.data?.items ?? [];
  const staffError = errors.fields.staffId;
  const pinError = errors.fields.pin;

  return (
    <SplitAuthLayout
      panelTitle="직원 로그인"
      panelDescription="이름을 고르고 개인 PIN을 입력하면 수강 사무 화면이 열립니다."
      cardTitle="배움마루 로그인"
      cardSubtitle="직원 이름을 고른 뒤 PIN을 넣어 주세요."
      cardFooter={footer}
    >
      {staffList.isPending ? (
        <NoticeBanner message="직원 목록을 불러오는 중…" />
      ) : staffList.isError ? (
        <NoticeBanner
          tone="error"
          message={staffList.error.message}
          actionLabel="다시 해 보기"
          onAction={() => void staffList.refetch()}
        />
      ) : (
        <form className={authStyles.form} onSubmit={onSubmit} noValidate>
          {items.length === 0 ? (
            <NoticeBanner
              tone="error"
              message="로그인할 수 있는 직원이 없습니다. 모든 계정이 사용 중지되었거나 유효기간이 지났습니다."
            />
          ) : (
            <fieldset
              className={authStyles.fieldset}
              aria-invalid={staffError !== undefined || undefined}
              aria-describedby={staffError ? "login-staff-error" : undefined}
            >
              <legend className={authStyles.legend}>이름</legend>
              {groupByRole(items).map((group) => (
                <div
                  key={group.role}
                  role="group"
                  aria-labelledby={`login-role-${group.role}`}
                  className={authStyles.roleGroup}
                >
                  <p
                    id={`login-role-${group.role}`}
                    className={authStyles.roleTitle}
                  >
                    {ROLE_LABELS[group.role]}
                  </p>
                  <div className={authStyles.names}>
                    {group.items.map((item) => (
                      <label key={item.id} className={authStyles.name}>
                        <input
                          type="radio"
                          name="staff"
                          value={item.id}
                          checked={staffId === item.id}
                          onChange={() => choose(item.id)}
                        />
                        {item.name}
                      </label>
                    ))}
                  </div>
                </div>
              ))}
              {staffError && (
                <p
                  id="login-staff-error"
                  className={authStyles.fieldError}
                  role="alert"
                >
                  {staffError}
                </p>
              )}
            </fieldset>
          )}

          <TextField
            id="login-pin"
            ref={pinInput}
            label="PIN"
            type="password"
            inputMode="numeric"
            autoComplete="current-password"
            maxLength={8}
            value={pin}
            error={pinError}
            onChange={(event) => {
              setPin(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, pin: undefined },
              }));
            }}
          />

          {errors.form && <NoticeBanner tone="error" message={errors.form} />}

          <div className={authStyles.actionsBetween}>
            <Button
              variant="secondary"
              onClick={onForgotPin}
              disabled={submit.isPending}
            >
              PIN을 잊었어요
            </Button>
            <Button
              type="submit"
              variant="primary"
              disabled={items.length === 0}
              busy={submit.isPending}
              busyLabel="확인하는 중…"
            >
              로그인
            </Button>
          </div>
        </form>
      )}
    </SplitAuthLayout>
  );
}
