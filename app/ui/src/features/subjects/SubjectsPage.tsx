import { useState } from "react";
import { useCan } from "../gate/auth/useCan";
import { ViewOnlyNotice } from "../gate/auth/ViewOnlyNotice";
import { useQuery } from "@tanstack/react-query";
import { listSubjects } from "../../api/subjects";
import formStyles from "../../components/form/form.module.css";
import styles from "../../components/layout/listPage.module.css";
import { SubjectForm } from "./SubjectForm";
import { SubjectList } from "./SubjectList";

/**
 * 과목 목록. 왼쪽은 과목 목록, 오른쪽은 입력 칸.
 *
 * 평소 오른쪽은 새 과목 등록이고, 목록에서 과목을 고르면 그 과목 고치기로 바뀐다.
 * 과목은 칸이 적어 목록에 받은 값으로 바로 고친다.
 * 기초 정보를 바꿀 수 없는 역할(BR-K3)에게는 입력 칸 대신 안내를 보여 준다.
 */
export function SubjectsPage() {
  const subjects = useQuery({ queryKey: ["subjects"], queryFn: listSubjects });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canEdit = useCan("editReferenceData");

  const selected =
    subjects.data?.items.find((subject) => subject.id === selectedId) ?? null;

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
      <SubjectList
        subjects={subjects}
        selectedId={selectedId}
        onSelect={select}
      />
      <aside className={styles.side}>
        {!canEdit ? (
          <ViewOnlyNotice />
        ) : subjects.data ? (
          // 다른 과목을 고르거나 등록으로 돌아오면 칸을 새로 만듦
          <SubjectForm
            key={selected?.id ?? "new"}
            subjects={subjects.data.items}
            editing={selected ?? undefined}
            initialNotice={notice}
            onClose={close}
          />
        ) : (
          <p className={formStyles.loading}>과목을 불러오는 중입니다…</p>
        )}
      </aside>
    </div>
  );
}
