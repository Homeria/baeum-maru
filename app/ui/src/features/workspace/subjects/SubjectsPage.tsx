import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { useCan } from "../../gate/auth/useCan";
import { listSubjects } from "./api";
import { SubjectForm } from "./SubjectForm";
import { SubjectList } from "./SubjectList";
import styles from "./SubjectsPage.module.css";

type SubjectDrawerState =
  null | { mode: "create" } | { mode: "edit"; id: number };

/**
 * 과목 목록. 전체 폭 표를 보여 주고, 등록과 고치기는 오른쪽 서랍에서 처리한다.
 *
 * 기초 정보를 바꿀 수 없는 역할(BR-K3)에게는 등록·고치기 버튼을 숨긴다.
 */
export function SubjectsPage() {
  const subjects = useQuery({ queryKey: ["subjects"], queryFn: listSubjects });
  const [drawer, setDrawer] = useState<SubjectDrawerState>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canEdit = useCan("editReferenceData");

  const selectedId = drawer?.mode === "edit" ? drawer.id : null;
  const selected =
    subjects.data?.items.find((subject) => subject.id === selectedId) ?? null;

  const finish = (nextNotice: string) => {
    setNotice(nextNotice);
    setDrawer(null);
  };

  return (
    <div className={styles.page}>
      {notice && <NoticeBanner message={notice} />}
      <SubjectList
        subjects={subjects}
        selectedId={selectedId}
        canEdit={canEdit}
        onSelect={(id) => {
          setNotice(null);
          setDrawer({ mode: "edit", id });
        }}
        onCreate={() => {
          setNotice(null);
          setDrawer({ mode: "create" });
        }}
      />

      {canEdit && drawer !== null && subjects.data && (
        <SubjectForm
          key={drawer.mode === "edit" ? drawer.id : "new"}
          subjects={subjects.data.items}
          editing={drawer.mode === "edit" ? (selected ?? undefined) : undefined}
          onSaved={finish}
          onDeleted={finish}
          onClose={() => setDrawer(null)}
        />
      )}
    </div>
  );
}
