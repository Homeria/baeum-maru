import { useState } from "react";
import styles from "../../components/form/form.module.css";
import authStyles from "./auth.module.css";

/** {@link RecoveryCodeNotice}의 속성. */
export interface RecoveryCodeNoticeProps {
  /** 4자씩 끊은 복구 코드. 예: `K7QF-M2XA-9TRE-HW4C` */
  code: string;
  /** 직원이 적어 두었다고 확인했다. */
  onDone: () => void;
}

/**
 * 복구 코드를 한 번 보여 주고 인쇄해 두라고 안내한다 (BR-K11, BR-K12).
 *
 * 코드는 다시 볼 수 없어서, 적어 두었다고 확인해야 다음으로 넘어간다.
 */
export function RecoveryCodeNotice({ code, onDone }: RecoveryCodeNoticeProps) {
  const [kept, setKept] = useState(false);
  const issuedOn = new Date().toLocaleDateString("ko-KR");

  return (
    <div className={authStyles.page}>
      <section
        className={`${styles.form} ${authStyles.card} ${authStyles.sheet}`}
        aria-labelledby="recovery-code-title"
      >
        <h2 id="recovery-code-title" className={styles.title}>
          배움마루 복구 코드
        </h2>
        <p className={authStyles.intro}>
          관리자가 PIN을 잊었을 때 이 코드로 PIN을 다시 정합니다.
        </p>

        <p className={authStyles.code} aria-label={`복구 코드 ${code}`}>
          {code}
        </p>

        <p className={`${authStyles.hint} ${authStyles.printOnly}`}>
          만든 날: {issuedOn}. 한 번 쓰면 새 코드가 나오고 이 코드는 못 씁니다.
        </p>
        <ol className={`${authStyles.steps} ${authStyles.printOnly}`}>
          <li>로그인 화면에서 "PIN을 잊었어요"를 누릅니다.</li>
          <li>관리자 이름을 고르고 이 코드와 새 PIN을 넣습니다.</li>
        </ol>

        <div className={`${authStyles.stack} ${authStyles.noPrint}`}>
          <p className={authStyles.warning}>
            이 코드는 지금만 보입니다. 인쇄하거나 적어서 다른 사람이 보지 않는
            곳(서랍, 금고 등)에 두세요.
          </p>
          <div className={styles.actions}>
            <button
              type="button"
              className={styles.secondary}
              onClick={() => window.print()}
            >
              인쇄
            </button>
          </div>
          <label className={styles.choice}>
            <input
              type="checkbox"
              checked={kept}
              onChange={(event) => setKept(event.target.checked)}
            />
            인쇄하거나 적어 두었습니다
          </label>
          <div className={styles.actions}>
            <button
              type="button"
              className={styles.submit}
              disabled={!kept}
              onClick={onDone}
            >
              다음
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}
