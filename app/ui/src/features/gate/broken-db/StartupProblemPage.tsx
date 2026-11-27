import type { StartupStateCode } from "../../../api/generated/StartupStateCode";
import styles from "../../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";

/** 복원해도 소용없는 문제의 제목과 안내. */
const MESSAGES: Record<
  Exclude<StartupStateCode, "ready" | "broken">,
  { title: string; body: string }
> = {
  newer_version: {
    title: "앱을 새 버전으로 바꿔 주세요",
    body: "이 데이터는 더 새 버전의 배움마루에서 쓰던 것입니다. 이 앱으로 열면 데이터가 상할 수 있어 열지 않았습니다. 새 버전을 받아 다시 켜 주세요.",
  },
  unavailable: {
    title: "데이터 폴더를 쓸 수 없습니다",
    body: "데이터 폴더를 만들거나 읽지 못했습니다. 포터블판이면 USB가 꽂혀 있는지, 폴더를 다른 프로그램이 쓰고 있지 않은지 확인한 뒤 앱을 다시 켜 주세요.",
  },
};

/** {@link StartupProblemPage}의 속성. */
export interface StartupProblemPageProps {
  state: Exclude<StartupStateCode, "ready" | "broken">;
}

/** 켤 때 DB를 쓸 수 없지만 복원으로 풀 일이 아닐 때의 안내(ADR 0011). */
export function StartupProblemPage({ state }: StartupProblemPageProps) {
  const message = MESSAGES[state];
  return (
    <div className={authStyles.page}>
      <section className={`${styles.form} ${authStyles.card}`}>
        <h2 className={styles.title}>{message.title}</h2>
        <p className={authStyles.intro} role="alert">
          {message.body}
        </p>
      </section>
    </div>
  );
}
