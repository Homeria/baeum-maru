import styles from "../../components/form/form.module.css";
import authStyles from "../auth/auth.module.css";

/** {@link RemotePage}의 속성. */
export interface RemotePageProps {
  /** 이 PC에 예전 DB가 남아 있다(ADR 0021). */
  leftoverData: boolean;
}

/** 접속 모드로 켰을 때의 화면. 호스트 주소 입력과 기기 승인은 이 화면에 붙는다. */
export function RemotePage({ leftoverData }: RemotePageProps) {
  return (
    <div className={authStyles.page}>
      <section className={`${styles.form} ${authStyles.card}`}>
        <h2 className={styles.title}>다른 PC에 접속</h2>
        <p className={authStyles.intro}>
          이 PC는 호스트에 접속해서 쓰도록 정해져 있습니다.
        </p>
        {leftoverData && (
          <p className={authStyles.hint}>
            이 PC에 이전 데이터가 남아 있습니다. 지우지 않았고, 접속 모드에서는
            쓰지 않습니다.
          </p>
        )}
      </section>
    </div>
  );
}
