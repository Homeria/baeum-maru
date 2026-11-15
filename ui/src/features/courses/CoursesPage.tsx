import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listTerms } from "../../api/terms";
import formStyles from "../../components/form/form.module.css";
import { TermBar } from "../terms/TermBar";
import { TermForm } from "../terms/TermForm";
import { currentTerm } from "../terms/termFormData";
import styles from "./CoursesPage.module.css";

/**
 * 강좌 탭. 맨 위에서 학기를 고르고, 그 아래에 그 학기의 강좌를 보여 준다.
 *
 * 처음에는 시작일이 가장 늦은 학기를 보여 준다. 학기가 하나도 없으면 학기 등록 칸부터 띄운다.
 */
export function CoursesPage() {
  const terms = useQuery({ queryKey: ["terms"], queryFn: listTerms });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  if (terms.isPending) {
    return <p className={formStyles.loading}>학기를 불러오는 중입니다…</p>;
  }
  if (terms.isError) {
    return (
      <p className={formStyles.formError} role="alert">
        학기를 불러오지 못했습니다. 다시 열어 주세요.
      </p>
    );
  }

  const current = currentTerm(terms.data.items, selectedId);
  if (!current) {
    return (
      <div className={styles.empty}>
        <p className={styles.emptyMessage}>
          강좌를 넣으려면 먼저 학기를 등록해 주세요.
        </p>
        <TermForm
          onSaved={(id, next) => {
            setNotice(next);
            setSelectedId(id);
          }}
        />
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <TermBar
        terms={terms.data.items}
        current={current}
        onSelect={setSelectedId}
        notice={notice}
        onNotice={setNotice}
      />
      <div className={styles.content}>
        {/* 강좌 목록과 입력 칸은 다음 작업에서 */}
        <p className={formStyles.loading}>강좌 목록은 준비 중입니다.</p>
      </div>
    </div>
  );
}
