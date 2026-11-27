import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { StaffResponse } from "../../../api/generated/StaffResponse";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { listStaff } from "./api";
import { PinResetForm } from "./PinResetForm";
import { StaffForm } from "./StaffForm";
import { StaffList } from "./StaffList";
import styles from "./StaffPage.module.css";

type StaffDrawerState =
  null | { mode: "create" } | { mode: "edit"; id: number };

/**
 * 직원 목록. 전체 폭 표를 보여 주고, 등록·고치기는 오른쪽 서랍, PIN 새로 정하기는 모달에서 처리한다.
 */
export function StaffPage() {
  const staff = useQuery({ queryKey: ["staff"], queryFn: listStaff });
  const [drawer, setDrawer] = useState<StaffDrawerState>(null);
  const [pinResetTarget, setPinResetTarget] = useState<StaffResponse | null>(
    null,
  );
  const [notice, setNotice] = useState<string | null>(null);

  const selectedId = drawer?.mode === "edit" ? drawer.id : null;
  const selected =
    staff.data?.items.find((item) => item.id === selectedId) ?? null;

  const finish = (nextNotice: string) => {
    setNotice(nextNotice);
    setDrawer(null);
    setPinResetTarget(null);
  };

  return (
    <div className={styles.page}>
      {notice && <NoticeBanner message={notice} />}

      <StaffList
        staff={staff}
        selectedId={selectedId}
        onSelect={(id) => {
          setNotice(null);
          setDrawer({ mode: "edit", id });
        }}
        onCreate={() => {
          setNotice(null);
          setDrawer({ mode: "create" });
        }}
        onResetPin={(target) => {
          setNotice(null);
          setPinResetTarget(target);
        }}
      />

      {drawer !== null && staff.data && (
        <StaffForm
          key={drawer.mode === "edit" ? drawer.id : "new"}
          editing={drawer.mode === "edit" ? (selected ?? undefined) : undefined}
          onSaved={finish}
          onClose={() => setDrawer(null)}
          onRequestPinReset={
            selected
              ? () => {
                  setDrawer(null);
                  setPinResetTarget(selected);
                }
              : undefined
          }
        />
      )}

      {pinResetTarget && (
        <PinResetForm
          key={pinResetTarget.id}
          staff={pinResetTarget}
          onSaved={finish}
          onClose={() => setPinResetTarget(null)}
        />
      )}
    </div>
  );
}
