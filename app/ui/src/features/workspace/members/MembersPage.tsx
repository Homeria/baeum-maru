import { useState } from "react";
import { useCan } from "../../gate/auth/useCan";
import { ViewOnlyNotice } from "../../gate/auth/ViewOnlyNotice";
import { MemberEditor } from "./MemberEditor";
import { MemberForm } from "./MemberForm";
import { MemberList } from "./MemberList";
import styles from "../../../components/layout/listPage.module.css";

/**
 * 회원 탭. 왼쪽은 찾기와 목록, 오른쪽은 입력 칸.
 *
 * 평소 오른쪽은 새 회원 등록이고, 목록에서 회원을 고르면 그 회원 고치기로 바뀐다.
 * 회원을 바꿀 수 없는 역할(BR-K3)에게는 입력 칸 대신 안내를 보여 준다.
 */
export function MembersPage() {
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canEdit = useCan("editMembers");

  const select = (id: number) => {
    setNotice(null);
    setSelectedId(id);
  };

  const close = (next?: string) => {
    setNotice(next ?? null);
    setSelectedId(null);
  };

  return (
    <div className={styles.page}>
      <MemberList selectedId={selectedId} onSelect={select} />
      <aside className={styles.side}>
        {!canEdit ? (
          <ViewOnlyNotice />
        ) : selectedId === null ? (
          <MemberForm initialNotice={notice} onClose={close} />
        ) : (
          // 다른 회원을 고르면 칸을 새로 만듦
          <MemberEditor key={selectedId} id={selectedId} onClose={close} />
        )}
      </aside>
    </div>
  );
}
