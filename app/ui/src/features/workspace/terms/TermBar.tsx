import { useId, useState } from "react";
import type { TermResponse } from "../../../api/generated/TermResponse";
import formStyles from "../../../components/form/form.module.css";
import { termPeriodLabel } from "./termFormData";
import { TermDialog } from "./TermDialog";
import styles from "./TermBar.module.css";

/** {@link TermBar}의 속성. */
export interface TermBarProps {
  /** 시작일이 늦은 학기부터. 하나 이상. */
  terms: readonly TermResponse[];
  /** 지금 보여 주는 학기. */
  current: TermResponse;
  /** 다른 학기를 골랐다. 새로 등록한 학기도 이것으로 고른다. */
  onSelect: (id: number) => void;
  /** 보여 줄 알림. 예: "등록했습니다: 2027년 2학기" */
  notice: string | null;
  onNotice: (notice: string | null) => void;
  /** 학기를 바꿀 수 있는 역할인지 (BR-K3). 아니면 고치기·새 학기 버튼을 숨긴다. */
  canEdit: boolean;
}

/**
 * 강좌 탭 맨 위의 학기 줄. 학기를 고르고, 새 학기를 등록하거나 지금 학기를 고친다.
 *
 * 등록과 고치기는 작은 창으로 띄워 아래 강좌 목록 자리를 건드리지 않는다.
 */
export function TermBar({
  terms,
  current,
  onSelect,
  notice,
  onNotice,
  canEdit,
}: TermBarProps) {
  const [dialog, setDialog] = useState<"new" | "edit" | null>(null);
  const selectId = useId();

  const open = (next: "new" | "edit") => {
    onNotice(null);
    setDialog(next);
  };

  return (
    <div className={styles.bar}>
      <label htmlFor={selectId} className={styles.label}>
        학기
      </label>
      <select
        id={selectId}
        className={styles.select}
        value={current.id}
        onChange={(event) => {
          onNotice(null);
          onSelect(Number(event.target.value));
        }}
      >
        {terms.map((term) => (
          <option key={term.id} value={term.id}>
            {term.name}
          </option>
        ))}
      </select>
      <span className={styles.period}>{termPeriodLabel(current)}</span>
      {canEdit && (
        <>
          <button
            type="button"
            className={formStyles.secondary}
            onClick={() => open("edit")}
          >
            학기 고치기
          </button>
          <button
            type="button"
            className={formStyles.secondary}
            onClick={() => open("new")}
          >
            새 학기
          </button>
        </>
      )}
      {notice && (
        <p className={formStyles.notice} role="status">
          {notice}
        </p>
      )}

      {dialog && (
        <TermDialog
          // 등록과 고치기를 오가면 칸을 새로 만듦
          key={dialog}
          editing={dialog === "edit" ? current : undefined}
          onSaved={(id, next) => {
            setDialog(null);
            onNotice(next);
            onSelect(id);
          }}
          onDeleted={(next) => {
            setDialog(null);
            onNotice(next);
          }}
          onCancel={() => setDialog(null)}
        />
      )}
    </div>
  );
}
