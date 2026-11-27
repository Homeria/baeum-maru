import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listStaff } from "./api";
import formStyles from "../../../components/form/form.module.css";
import styles from "../../../components/layout/listPage.module.css";
import { PinResetForm } from "./PinResetForm";
import { StaffForm } from "./StaffForm";
import { StaffList } from "./StaffList";
import pageStyles from "./StaffPage.module.css";

/**
 * 직원 목록. 왼쪽은 직원 목록, 오른쪽은 입력 칸.
 *
 * 평소 오른쪽은 새 직원 등록이고, 목록에서 직원을 고르면 그 직원 고치기와 PIN 새로 정하기가 나온다.
 */
export function StaffPage() {
  const staff = useQuery({ queryKey: ["staff"], queryFn: listStaff });
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const selected =
    staff.data?.items.find((item) => item.id === selectedId) ?? null;

  return (
    <div className={styles.page}>
      <StaffList
        staff={staff}
        selectedId={selectedId}
        onSelect={setSelectedId}
      />
      <aside className={styles.side}>
        {staff.data ? (
          <div className={pageStyles.forms}>
            {/* 다른 직원을 고르거나 등록으로 돌아오면 칸을 새로 만듦 */}
            <StaffForm
              key={selected?.id ?? "new"}
              editing={selected ?? undefined}
              onClose={() => setSelectedId(null)}
            />
            {selected && <PinResetForm key={selected.id} staff={selected} />}
          </div>
        ) : (
          <p className={formStyles.loading}>직원을 불러오는 중입니다…</p>
        )}
      </aside>
    </div>
  );
}
