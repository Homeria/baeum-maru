import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { changeRunMode } from "../run-mode/api";
import { ConfirmDialog } from "../../../components/dialog/ConfirmDialog";
import formStyles from "../../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";
import { CancelHandoverDialog } from "./CancelHandoverDialog";

/**
 * 호스트를 넘긴 PC의 화면(BR-H1). 접수를 받지 않고, 이 PC의 데이터는 지우지 않는다.
 *
 * 다른 PC에 접속하는 모드로 바꾸거나, 관리자 PIN으로 넘기기를 취소한다(BR-H2).
 */
export function HandedOverPage() {
  const [dialog, setDialog] = useState<"remote" | "cancel" | null>(null);
  const toRemote = useMutation({
    mutationFn: () => changeRunMode("remote"),
  });

  return (
    <div className={authStyles.page}>
      <section className={`${formStyles.form} ${authStyles.card}`}>
        <h2 className={formStyles.title}>이 PC는 호스트를 넘겼습니다</h2>
        <p className={authStyles.intro}>
          새 호스트 PC에서 배움마루를 써 주세요. 이 PC도 계속 쓰려면 새 호스트
          PC에 접속하는 모드로 바꿔 주세요.
        </p>
        <p className={authStyles.hint}>
          이 PC의 데이터는 지우지 않았고, 쓰지도 않습니다.
        </p>
        <div className={formStyles.actions}>
          <button
            type="button"
            className={formStyles.secondary}
            onClick={() => setDialog("cancel")}
          >
            넘기기 취소
          </button>
          <button
            type="button"
            className={formStyles.submit}
            onClick={() => setDialog("remote")}
          >
            다른 PC에 접속하는 모드로 바꾸기
          </button>
        </div>
      </section>
      {dialog === "remote" && (
        <ConfirmDialog
          title="이 PC를 다른 PC에 접속하는 모드로 바꿀까요?"
          warning="새 호스트 PC의 주소를 넣고, 그 호스트에서 이 PC를 승인받아야 합니다. 이 PC의 데이터는 지우지 않습니다."
          confirmLabel="바꾸고 다시 켜기"
          busyLabel="바꾸는 중…"
          busy={toRemote.isPending}
          onConfirm={() => toRemote.mutate()}
          onCancel={() => setDialog(null)}
        >
          {toRemote.isError && (
            <p className={formStyles.formError} role="alert">
              {toRemote.error.message}
            </p>
          )}
        </ConfirmDialog>
      )}
      {dialog === "cancel" && (
        <CancelHandoverDialog onCancel={() => setDialog(null)} />
      )}
    </div>
  );
}
