import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { useCan } from "../../gate/auth/useCan";
import { listPriorityReasons } from "./api";
import { PriorityReasonForm } from "./PriorityReasonForm";
import { PriorityReasonList } from "./PriorityReasonList";
import styles from "./PriorityReasonsPage.module.css";

type ReasonDrawerState =
  null | { mode: "create" } | { mode: "edit"; id: number };

/**
 * 우선 배정 사유 목록(BR-S10). 전체 폭 표를 보여 주고, 등록과 고치기는 오른쪽 서랍에서 처리한다.
 *
 * 목록은 모두 보고, 바꾸기는 관리자만 한다. 다른 역할에게는 안내 배너만 띄우고 편집 버튼을 숨긴다.
 */
export function PriorityReasonsPage() {
  const reasons = useQuery({
    queryKey: ["priority-reasons"],
    queryFn: listPriorityReasons,
  });
  const [drawer, setDrawer] = useState<ReasonDrawerState>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const canManage = useCan("managePriorityReasons");

  const selectedId = drawer?.mode === "edit" ? drawer.id : null;
  const selected =
    reasons.data?.items.find((reason) => reason.id === selectedId) ?? null;

  const finish = (nextNotice: string) => {
    setNotice(nextNotice);
    setDrawer(null);
  };

  return (
    <div className={styles.page}>
      {!canManage && (
        <NoticeBanner message="우선 배정 사유는 관리자만 등록하거나 바꿀 수 있습니다." />
      )}
      {notice && <NoticeBanner message={notice} />}

      <PriorityReasonList
        reasons={reasons}
        selectedId={selectedId}
        canManage={canManage}
        onSelect={
          canManage
            ? (id) => {
                setNotice(null);
                setDrawer({ mode: "edit", id });
              }
            : undefined
        }
        onCreate={
          canManage
            ? () => {
                setNotice(null);
                setDrawer({ mode: "create" });
              }
            : undefined
        }
      />

      {canManage && drawer !== null && (
        <PriorityReasonForm
          key={drawer.mode === "edit" ? drawer.id : "new"}
          editing={drawer.mode === "edit" ? (selected ?? undefined) : undefined}
          onSaved={finish}
          onDeleted={finish}
          onClose={() => setDrawer(null)}
        />
      )}
    </div>
  );
}
