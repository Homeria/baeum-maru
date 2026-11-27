import { useState } from "react";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { useCan } from "../../gate/auth/useCan";
import { MemberEditor } from "./MemberEditor";
import { MemberForm } from "./MemberForm";
import { MemberList } from "./MemberList";
import styles from "./MembersPage.module.css";

type MemberDrawerState =
  null | { mode: "create" } | { mode: "edit"; id: number };

/**
 * 회원 탭. 전체 폭 검색·목록 표를 보여 주고, 등록과 정보 고치기는 오른쪽 서랍에서 처리한다.
 *
 * 회원을 바꿀 수 없는 역할(BR-K3)에게는 등록·고치기 동작을 숨긴다.
 */
export function MembersPage() {
  const [drawer, setDrawer] = useState<MemberDrawerState>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canEdit = useCan("editMembers");

  const openCreate = () => {
    setNotice(null);
    setDrawer({ mode: "create" });
  };

  const openEdit = (id: number) => {
    if (!canEdit) {
      return;
    }
    setNotice(null);
    setDrawer({ mode: "edit", id });
  };

  const finish = (nextNotice: string) => {
    setNotice(nextNotice);
    setDrawer(null);
  };

  const closeDrawer = () => {
    setDrawer(null);
  };

  const selectedId = drawer?.mode === "edit" ? drawer.id : null;

  return (
    <div className={styles.page}>
      {notice && <NoticeBanner message={notice} />}
      <MemberList
        selectedId={selectedId}
        canEdit={canEdit}
        onSelect={canEdit ? openEdit : undefined}
        onCreate={canEdit ? openCreate : undefined}
      />

      {canEdit && drawer?.mode === "create" && (
        <MemberForm onSaved={finish} onClose={closeDrawer} />
      )}
      {canEdit && drawer?.mode === "edit" && (
        <MemberEditor
          key={drawer.id}
          id={drawer.id}
          onSaved={finish}
          onDeleted={finish}
          onClose={closeDrawer}
        />
      )}
    </div>
  );
}
