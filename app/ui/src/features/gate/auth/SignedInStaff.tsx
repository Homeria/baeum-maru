import { useState } from "react";
import { logout } from "./api";
import type { Session } from "../../../api/session";
import formStyles from "../../../components/form/form.module.css";
import { ROLE_LABELS } from "./roles";
import styles from "./SignedInStaff.module.css";

/** {@link SignedInStaff}의 속성. */
export interface SignedInStaffProps {
  session: Session;
}

/** 화면 위쪽: 로그인한 직원의 이름·역할과 로그아웃 버튼. */
export function SignedInStaff({ session }: SignedInStaffProps) {
  const [leaving, setLeaving] = useState(false);
  const { name, role } = session.staff;

  return (
    <div className={styles.staff}>
      <span>
        <strong>{name}</strong> ({ROLE_LABELS[role]})
      </span>
      <button
        type="button"
        className={formStyles.secondary}
        disabled={leaving}
        onClick={() => {
          setLeaving(true);
          // 끝나면 세션이 비어 앱이 로그인 화면으로 바뀐다
          void logout();
        }}
      >
        {leaving ? "로그아웃하는 중…" : "로그아웃"}
      </button>
    </div>
  );
}
