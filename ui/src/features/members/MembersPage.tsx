import { MemberForm } from "./MemberForm";
import { MemberList } from "./MemberList";
import styles from "./MembersPage.module.css";

/** 회원 탭. 왼쪽은 찾기와 목록, 오른쪽은 입력 칸. */
export function MembersPage() {
  return (
    <div className={styles.page}>
      <MemberList />
      <aside className={styles.side}>
        <MemberForm />
      </aside>
    </div>
  );
}
