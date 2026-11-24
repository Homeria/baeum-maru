import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { fetchSetupStatus } from "../api/setup";
import { startupState } from "../api/startup";
import { LockScreen } from "../features/auth/LockScreen";
import { LoginPage } from "../features/auth/LoginPage";
import { RecoveryCodeNotice } from "../features/auth/RecoveryCodeNotice";
import { RecoveryPage } from "../features/auth/RecoveryPage";
import { SetupPage } from "../features/auth/SetupPage";
import { useIdleLock } from "../features/auth/useIdleLock";
import { useSession } from "../features/auth/useSession";
import { BrokenDbPage } from "../features/broken-db/BrokenDbPage";
import { StartupProblemPage } from "../features/broken-db/StartupProblemPage";
import { ChooseRunModePage } from "../features/run-mode/ChooseRunModePage";
import { RemotePage } from "../features/run-mode/RemotePage";
import { useRunMode } from "../features/run-mode/useRunMode";
import formStyles from "../components/form/form.module.css";
import { MainScreen } from "./MainScreen";

/**
 * 앱의 첫 갈림길: 실행 모드 고르기, DB 복원, 처음 설정, 로그인, 메인 화면 중 하나를 띄운다.
 *
 * 맨 먼저 이 PC의 실행 모드를 앱에 묻는다(ADR 0021). 호스트면 DB를 쓸 수 있는지 묻는다(ADR 0011). 깨졌으면 로그인 없이 복원 화면을 띄운다.
 * 처음 설정을 마치거나 복구 코드로 PIN을 다시 정하면 새 복구 코드 안내를 먼저 보여 준다.
 */
export function App() {
  const runMode = useRunMode();
  // 앱에 묻지 못했으면(예: 앱 밖 브라우저) 호스트로 본다
  const host = runMode.data?.mode === "host" || runMode.isError;
  const startup = useQuery({
    queryKey: ["startupState"],
    queryFn: startupState,
    enabled: host,
    staleTime: Infinity,
  });
  // 앱에 묻지 못했으면 평소처럼 호스트에 묻는다
  const ready = startup.data?.state === "ready" || startup.isError;
  const status = useQuery({
    queryKey: ["setupStatus"],
    queryFn: fetchSetupStatus,
    enabled: ready,
  });
  // 처음 설정이나 복구에서 받은 복구 코드. 안내를 닫을 때까지만 메모리에 둔다
  const [recoveryCode, setRecoveryCode] = useState<string | null>(null);
  // 로그인 화면에서 "PIN을 잊었어요"를 눌렀다
  const [recovering, setRecovering] = useState(false);
  const session = useSession();
  useIdleLock(session !== null && !session.locked);

  if (recoveryCode !== null) {
    return (
      <RecoveryCodeNotice
        code={recoveryCode}
        onDone={() => setRecoveryCode(null)}
      />
    );
  }
  if (runMode.isPending) {
    return <p className={formStyles.loading}>앱을 준비하는 중…</p>;
  }
  if (runMode.data !== undefined && runMode.data.mode === null) {
    return <ChooseRunModePage />;
  }
  if (runMode.data?.mode === "remote") {
    return <RemotePage leftoverData={runMode.data.leftoverData} />;
  }
  if (startup.isPending) {
    return <p className={formStyles.loading}>데이터를 확인하는 중…</p>;
  }
  if (startup.data !== undefined && startup.data.state !== "ready") {
    return startup.data.state === "broken" ? (
      <BrokenDbPage startup={startup.data} />
    ) : (
      <StartupProblemPage state={startup.data.state} />
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
  if (status.data.needed) {
    return <SetupPage onDone={setRecoveryCode} />;
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
    return <LoginPage onForgotPin={() => setRecovering(true)} />;
  }
  return (
    <>
      <MainScreen session={session} />
      {session.locked && <LockScreen session={session} />}
    </>
  );
}
