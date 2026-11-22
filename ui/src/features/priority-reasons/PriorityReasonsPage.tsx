import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listPriorityReasons } from "../../api/priorityReasons";
import formStyles from "../../components/form/form.module.css";
import styles from "../../components/layout/listPage.module.css";
import { useCan } from "../auth/useCan";
import { PriorityReasonForm } from "./PriorityReasonForm";
import { PriorityReasonList } from "./PriorityReasonList";

/**
 * 우선 배정 사유 목록(BR-S10). 왼쪽은 목록, 오른쪽은 입력 칸.
 *
 * 목록은 모두 보고, 바꾸기는 관리자만 한다. 다른 역할에게는 입력 칸 대신 안내를 보여 준다.
 */
export function PriorityReasonsPage() {
  const reasons = useQuery({
    queryKey: ["priority-reasons"],
    queryFn: listPriorityReasons,
  });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canManage = useCan("managePriorityReasons");

  const selected =
    reasons.data?.items.find((reason) => reason.id === selectedId) ?? null;

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
      <PriorityReasonList
        reasons={reasons}
        selectedId={selectedId}
        onSelect={canManage ? select : undefined}
      />
      <aside className={styles.side}>
        {!canManage ? (
          <div className={formStyles.form}>
            <p className={formStyles.notice} role="status">
              우선 배정 사유는 관리자만 바꿀 수 있습니다.
            </p>
          </div>
        ) : (
          // 다른 사유를 고르거나 등록으로 돌아오면 칸을 새로 만듦
          <PriorityReasonForm
            key={selected?.id ?? "new"}
            editing={selected ?? undefined}
            initialNotice={notice}
            onClose={close}
          />
        )}
      </aside>
    </div>
  );
}
