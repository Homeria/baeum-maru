import { useState } from "react";
import {
  SplitAuthLayout,
  type SplitAuthStep,
} from "../../../components/shell/SplitAuthLayout";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { Checkbox } from "../../../components/ui/Checkbox";
import authStyles from "./auth.module.css";

const SETUP_RECOVERY_STEPS: readonly SplitAuthStep[] = [
  { label: "관리자 계정 만들기", status: "done" },
  { label: "복구 코드 보관", status: "current" },
  { label: "기초 정보 미리 넣기", status: "upcoming" },
];

const PIN_RECOVERY_STEPS: readonly SplitAuthStep[] = [
  { label: "복구 코드로 PIN 다시 정하기", status: "done" },
  { label: "새 복구 코드 보관", status: "current" },
];

/** {@link RecoveryCodeNotice}의 속성. */
export interface RecoveryCodeNoticeProps {
  /** 4자씩 끊은 복구 코드. 예: `K7QF-M2XA-9TRE-HW4C` */
  code: string;
  /** 처음 설정 중인지, PIN 복구 중인지. 기본값은 `"setup"`. */
  mode?: "setup" | "recovery";
  /** 직원이 적어 두었다고 확인했다. */
  onDone: () => void;
}

/**
 * 복구 코드를 한 번 보여 주고 인쇄해 두라고 안내한다 (BR-K11, BR-K12).
 *
 * 코드는 다시 볼 수 없어서, 적어 두었다고 확인해야 다음으로 넘어간다.
 */
export function RecoveryCodeNotice({
  code,
  mode = "setup",
  onDone,
}: RecoveryCodeNoticeProps) {
  const [kept, setKept] = useState(false);
  const issuedOn = new Date().toLocaleDateString("ko-KR");
  const isSetup = mode === "setup";

  return (
    <SplitAuthLayout
      panelTitle={isSetup ? "처음 설정" : "PIN 복구"}
      panelDescription="관리자가 PIN을 잊었을 때 이 코드로 PIN을 다시 정할 수 있습니다."
      steps={isSetup ? SETUP_RECOVERY_STEPS : PIN_RECOVERY_STEPS}
      cardTitle="배움마루 복구 코드"
      cardSubtitle="관리자가 PIN을 잊었을 때 이 코드로 PIN을 다시 정합니다."
    >
      <section className={`${authStyles.stack} ${authStyles.sheet}`}>
        <p className={authStyles.code} aria-label={`복구 코드 ${code}`}>
          {code}
        </p>

        <p className={`${authStyles.hint} ${authStyles.printOnly}`}>
          만든 날: {issuedOn}. 한 번 쓰면 새 코드가 나오고 이 코드는 못 씁니다.
        </p>
        <ol className={`${authStyles.steps} ${authStyles.printOnly}`}>
          <li>로그인 화면에서 &quot;PIN을 잊었어요&quot;를 누릅니다.</li>
          <li>관리자 이름을 고르고 이 코드와 새 PIN을 넣습니다.</li>
        </ol>

        <div className={`${authStyles.stack} ${authStyles.noPrint}`}>
          <NoticeBanner
            tone="error"
            title="이 코드는 지금만 보입니다"
            message="인쇄하거나 적어서 다른 사람이 보지 않는 곳(서랍, 금고 등)에 두세요."
          />

          <Checkbox
            label="인쇄하거나 적어 두었습니다"
            checked={kept}
            onChange={(event) => setKept(event.target.checked)}
          />

          <div className={authStyles.actionsBetween}>
            <Button variant="secondary" onClick={() => window.print()}>
              인쇄
            </Button>
            <Button variant="primary" disabled={!kept} onClick={onDone}>
              다음
            </Button>
          </div>
        </div>
      </section>
    </SplitAuthLayout>
  );
}
