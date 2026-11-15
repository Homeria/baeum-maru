import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { fetchSetupStatus } from "../api/setup";
import { LockScreen } from "../features/auth/LockScreen";
import { LoginPage } from "../features/auth/LoginPage";
import { RecoveryCodeNotice } from "../features/auth/RecoveryCodeNotice";
import { SetupPage } from "../features/auth/SetupPage";
import { useIdleLock } from "../features/auth/useIdleLock";
import { useSession } from "../features/auth/useSession";
import formStyles from "../components/form/form.module.css";
import { MainScreen } from "./MainScreen";

/**
 * 앱의 첫 갈림길: 처음 설정, 로그인, 메인 화면 중 하나를 띄운다.
 *
 * 처음 설정을 마치면 복구 코드 안내를 먼저 보여 준다.
 */
export function App() {
  const status = useQuery({
    queryKey: ["setupStatus"],
    queryFn: fetchSetupStatus,
  });
  // 처음 설정에서 받은 복구 코드. 안내를 닫을 때까지만 메모리에 둔다
  const [recoveryCode, setRecoveryCode] = useState<string | null>(null);
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
    return <LoginPage />;
  }
  return (
    <>
      <MainScreen session={session} />
      {session.locked && <LockScreen session={session} />}
    </>
  );
}
