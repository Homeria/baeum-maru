import { useRef, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { changeRunMode } from "../../../api/runMode";
import { ModalDialog } from "../../../components/overlay/ModalDialog";
import { SplitAuthLayout } from "../../../components/shell/SplitAuthLayout";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import authStyles from "../auth/auth.module.css";
import { CancelHandoverDialog } from "./CancelHandoverDialog";

/**
 * 호스트를 넘긴 PC의 화면(BR-H1). 접수를 받지 않고, 이 PC의 데이터는 지우지 않는다.
 *
 * 다른 PC에 접속하는 모드로 바꾸거나, 관리자 PIN으로 넘기기를 취소한다(BR-H2).
 */
export function HandedOverPage() {
  const [dialog, setDialog] = useState<"remote" | "cancel" | null>(null);
  const cancelRemoteRef = useRef<HTMLButtonElement>(null);
  const toRemote = useMutation({
    mutationFn: () => changeRunMode("remote"),
  });

  return (
    <SplitAuthLayout
      panelTitle="호스트 넘김 완료"
      panelDescription="이 PC의 데이터는 지우지 않았고, 접수는 받지 않습니다. 이 PC도 계속 쓰려면 새 호스트에 접속하는 모드로 바꿔 주세요."
      cardTitle="이 PC는 호스트를 넘겼습니다"
      cardSubtitle="새 호스트 PC에서 배움마루를 써 주세요. 이 PC도 계속 쓰려면 새 호스트 PC에 접속하는 모드로 바꿔 주세요."
    >
      <div className={authStyles.stack}>
        <NoticeBanner message="이 PC의 데이터는 지우지 않았고, 쓰지도 않습니다." />
        <div className={authStyles.actionsBetween}>
          <Button variant="secondary" onClick={() => setDialog("cancel")}>
            넘기기 취소
          </Button>
          <Button variant="primary" onClick={() => setDialog("remote")}>
            다른 PC에 접속하는 모드로 바꾸기
          </Button>
        </div>
      </div>

      <ModalDialog
        open={dialog === "remote"}
        size="sm"
        title="이 PC를 다른 PC에 접속하는 모드로 바꿀까요?"
        busy={toRemote.isPending}
        initialFocusRef={cancelRemoteRef}
        onClose={() => setDialog(null)}
        footer={
          <>
            <Button
              ref={cancelRemoteRef}
              variant="secondary"
              disabled={toRemote.isPending}
              onClick={() => setDialog(null)}
            >
              그만두기
            </Button>
            <Button
              variant="primary"
              busy={toRemote.isPending}
              busyLabel="바꾸는 중…"
              onClick={() => toRemote.mutate()}
            >
              바꾸고 다시 켜기
            </Button>
          </>
        }
      >
        <div className={authStyles.stack}>
          <p className={authStyles.intro}>
            새 호스트 PC의 주소를 넣고, 그 호스트에서 이 PC를 승인받아야 합니다.
            이 PC의 데이터는 지우지 않습니다.
          </p>
          {toRemote.isError && (
            <NoticeBanner tone="error" message={toRemote.error.message} />
          )}
        </div>
      </ModalDialog>

      {dialog === "cancel" && (
        <CancelHandoverDialog onCancel={() => setDialog(null)} />
      )}
    </SplitAuthLayout>
  );
}
