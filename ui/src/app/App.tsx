import { useQuery } from "@tanstack/react-query";
import { startupState } from "../api/startup";
import { BrokenDbPage } from "../features/broken-db/BrokenDbPage";
import { StartupProblemPage } from "../features/broken-db/StartupProblemPage";
import { RemoteGate } from "../features/remote/RemoteGate";
import { ChooseRunModePage } from "../features/run-mode/ChooseRunModePage";
import { useRunMode } from "../features/run-mode/useRunMode";
import formStyles from "../components/form/form.module.css";
import { StaffFlow } from "./StaffFlow";

/**
 * 앱의 첫 갈림길: 실행 모드 고르기, 다른 PC에 접속, DB 복원 중 하나를 거쳐 로그인으로 간다.
 *
 * 맨 먼저 이 PC의 실행 모드를 앱에 묻는다(ADR 0021).
 * - 아직 고르지 않았으면 고르게 한다.
 * - 접속 모드면 호스트 주소와 기기 승인을 거친다(ADR 0004, BR-K15).
 * - 호스트면 DB를 쓸 수 있는지 묻는다(ADR 0011). 깨졌으면 로그인 없이 복원 화면을 띄운다.
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

  if (runMode.isPending) {
    return <p className={formStyles.loading}>앱을 준비하는 중…</p>;
  }
  if (runMode.data !== undefined && runMode.data.mode === null) {
    return <ChooseRunModePage />;
  }
  if (runMode.data?.mode === "remote") {
    return (
      <RemoteGate leftoverData={runMode.data.leftoverData}>
        <StaffFlow />
      </RemoteGate>
    );
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
  return <StaffFlow />;
}
