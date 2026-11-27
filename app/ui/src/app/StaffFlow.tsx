import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { SplitAuthLayout } from "../components/shell/SplitAuthLayout";
import { NoticeBanner } from "../components/surface/NoticeBanner";
import { fetchSetupStatus } from "../features/gate/auth/setupApi";
import { LockScreen } from "../features/gate/auth/LockScreen";
import { LoginPage } from "../features/gate/auth/LoginPage";
import { RecoveryCodeNotice } from "../features/gate/auth/RecoveryCodeNotice";
import { RecoveryPage } from "../features/gate/auth/RecoveryPage";
import { SetupBasicsPage } from "../features/gate/auth/SetupBasicsPage";
import { SetupPage } from "../features/gate/auth/SetupPage";
import { useIdleLock } from "../features/gate/auth/useIdleLock";
import { useSession } from "../features/gate/auth/useSession";
import { RunModeSwitch } from "../features/gate/run-mode/RunModeSwitch";
import { useRunMode } from "../features/gate/run-mode/useRunMode";
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
  const [recoveryNotice, setRecoveryNotice] = useState<{
    code: string;
    mode: "setup" | "recovery";
    nextBasics: boolean;
  } | null>(null);
  // 처음 설정 3단계(기초 정보 미리 넣기)를 진행 중인지
  const [settingUpBasics, setSettingUpBasics] = useState(false);
  // 로그인 화면에서 "PIN을 잊었어요"를 눌렀다
  const [recovering, setRecovering] = useState(false);
  const session = useSession();
  useIdleLock(session !== null && !session.locked);
  const runMode = useRunMode();
  const mode = runMode.data?.mode ?? null;

  if (recoveryNotice !== null) {
    return (
      <RecoveryCodeNotice
        code={recoveryNotice.code}
        mode={recoveryNotice.mode}
        onDone={() => {
          const goBasics = recoveryNotice.nextBasics;
          setRecoveryNotice(null);
          if (goBasics) {
            setSettingUpBasics(true);
          }
        }}
      />
    );
  }
  if (status.isPending) {
    return (
      <SplitAuthLayout
        panelTitle="호스트 연결"
        panelDescription="호스트 PC와 연결 상태를 확인하고 있습니다."
        cardTitle="호스트에 연결하는 중"
      >
        <NoticeBanner message="호스트에 연결하는 중…" />
      </SplitAuthLayout>
    );
  }
  if (status.isError) {
    return (
      <SplitAuthLayout
        panelTitle="호스트 연결"
        panelDescription="호스트 PC에 연결할 수 없으면 네트워크나 호스트 실행 상태를 확인해 주세요."
        cardTitle="호스트에 연결하지 못했습니다"
      >
        <NoticeBanner
          tone="error"
          message={status.error.message}
          actionLabel="다시 해 보기"
          onAction={() => void status.refetch()}
        />
      </SplitAuthLayout>
    );
  }
  // 로그인 전 화면 카드 아래의 실행 모드 바꾸기(ADR 0021). 직원이 없으면 지킬 데이터가 없다
  // 넘기기 상태는 이 화면까지 오지 않는다(BR-H1)
  const modeSwitch = (mode === "host" || mode === "remote") && (
    <RunModeSwitch current={mode} needsPin={!status.data.needed} />
  );
  if (status.data.needed) {
    return (
      <SetupPage
        onDone={({ recoveryCode, autoSignedIn }) =>
          setRecoveryNotice({
            code: recoveryCode,
            mode: "setup",
            nextBasics: autoSignedIn,
          })
        }
        footer={modeSwitch}
      />
    );
  }
  if (session === null) {
    if (recovering) {
      return (
        <RecoveryPage
          onBack={() => setRecovering(false)}
          onDone={(code) => {
            setRecovering(false);
            setRecoveryNotice({ code, mode: "recovery", nextBasics: false });
          }}
        />
      );
    }
    return (
      <LoginPage onForgotPin={() => setRecovering(true)} footer={modeSwitch} />
    );
  }
  if (settingUpBasics) {
    return (
      <>
        <SetupBasicsPage onDone={() => setSettingUpBasics(false)} />
        {session.locked && <LockScreen session={session} />}
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
