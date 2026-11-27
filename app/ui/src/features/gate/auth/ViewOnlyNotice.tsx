import styles from "../../../components/form/form.module.css";

/** 역할상 바꿀 수 없을 때 입력 칸 자리에 보여 준다 (BR-K3). */
export function ViewOnlyNotice() {
  return (
    <div className={styles.form}>
      <p className={styles.notice} role="status">
        지금 역할로는 보기만 할 수 있습니다. 바꿔야 하면 직원이나 관리자에게
        부탁해 주세요.
      </p>
    </div>
  );
}
