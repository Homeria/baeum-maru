import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { fetchSetupStatus } from "../api/setup";
import { LockScreen } from "../features/auth/LockScreen";
import { LoginPage } from "../features/auth/LoginPage";
import { RecoveryCodeNotice } from "../features/auth/RecoveryCodeNotice";
import { RecoveryPage } from "../features/auth/RecoveryPage";
import { SetupPage } from "../features/auth/SetupPage";
import { useIdleLock } from "../features/auth/useIdleLock";
import { useSession } from "../features/auth/useSession";
import { RunModeSwitch } from "../features/run-mode/RunModeSwitch";
import { useRunMode } from "../features/run-mode/useRunMode";
import formStyles from "../components/form/form.module.css";
import { MainScreen } from "./MainScreen";

/**
 * 호스트에 이어진 뒤의 흐름: 처음 설정, 로그인, 메인 화면 중 하나를 띄운다.
 *
 * 처음 설정을 마치거나 복구 코드로 PIN을 다시 정하면 새 복구 코드 안내를 먼저 보여 준다.
 */
export function StaffFlow() {
  const status = useQuery({
    queryKey: ["setupStatus"],
    queryFn: fetchSetupStatus,
  });
  // 처음 설정이나 복구에서 받은 복구 코드. 안내를 닫을 때까지만 메모리에 둔다
  const [recoveryCode, setRecoveryCode] = useState<string | null>(null);
  // 로그인 화면에서 "PIN을 잊었어요"를 눌렀다
  const [recovering, setRecovering] = useState(false);
  const session = useSession();
  useIdleLock(session !== null && !session.locked);
  const runMode = useRunMode();
  const mode = runMode.data?.mode ?? null;

  if (recoveryCode !== null) {
    return (
      <RecoveryCodeNotice
        code={recoveryCode}
        onDone={() => setRecoveryCode(null)}
      />
    );
  }
  if (status.isPending) {
    return <p className={formStyles.loading}>호스트에 연결하는 중…</p>;
  }
  if (status.isError) {
    return (
      <div className={formStyles.loading}>
        <p className={formStyles.formError} role="alert">
          {status.error.message}
        </p>
        <button
          type="button"
          className={formStyles.secondary}
          onClick={() => void status.refetch()}
        >
          다시 해 보기
        </button>
      </div>
    );
  }
  // 로그인 전 화면 구석의 실행 모드 바꾸기(ADR 0021). 직원이 없으면 지킬 데이터가 없다
  const modeSwitch = mode !== null && (
    <RunModeSwitch current={mode} needsPin={!status.data.needed} />
  );
  if (status.data.needed) {
    return (
      <>
        <SetupPage onDone={setRecoveryCode} />
        {modeSwitch}
      </>
    );
  }
  if (session === null) {
    if (recovering) {
      return (
        <RecoveryPage
          onBack={() => setRecovering(false)}
          onDone={(code) => {
            setRecovering(false);
            setRecoveryCode(code);
          }}
        />
      );
    }
    return (
      <>
        <LoginPage onForgotPin={() => setRecovering(true)} />
        {modeSwitch}
      </>
    );
  }
  return (
    <>
      <MainScreen session={session} />
      {session.locked && <LockScreen session={session} />}
    </>
  );
}
