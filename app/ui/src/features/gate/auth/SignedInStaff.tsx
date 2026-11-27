import { useState } from "react";
import { lockSession, type Session } from "../../../api/session";
import { logout } from "./api";
import { ROLE_LABELS } from "./roles";
import styles from "./SignedInStaff.module.css";

/** {@link SignedInStaff}의 속성. */
export interface SignedInStaffProps {
  session: Session;
}

/** 왼쪽 메뉴 맨 아래: 로그인한 직원의 이름·역할과 잠금·로그아웃 버튼. */
export function SignedInStaff({ session }: SignedInStaffProps) {
  const [leaving, setLeaving] = useState(false);
  const { name, role } = session.staff;

  return (
    <div className={styles.staff}>
      <div className={styles.identity}>
        <strong className={styles.name}>{name}</strong>
        <span className={styles.role}>{ROLE_LABELS[role]}</span>
      </div>
      <div className={styles.actions}>
        <button
          type="button"
          className={styles.actionButton}
          disabled={leaving}
          onClick={() => lockSession()}
        >
          화면 잠금
        </button>
        <button
          type="button"
          className={styles.actionButton}
          disabled={leaving}
          onClick={() => {
            setLeaving(true);
            // 끝나면 세션이 비어 앱이 로그인 화면으로 바뀐다
            void logout();
          }}
        >
          {leaving ? "나가는 중…" : "로그아웃"}
        </button>
      </div>
    </div>
  );
}
