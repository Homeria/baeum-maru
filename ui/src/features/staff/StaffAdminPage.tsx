import { useState } from "react";
import { Tabs, type TabItem } from "../../components/Tabs/Tabs";
import { AccessLogList } from "../access-logs/AccessLogList";
import styles from "../reference-data/ReferenceDataPage.module.css";
import { StaffPage } from "./StaffPage";

type ListKey = "staff" | "accessLogs";

const LISTS: readonly TabItem<ListKey>[] = [
  { key: "staff", label: "직원" },
  { key: "accessLogs", label: "접속 기록" },
];

/** 직원 관리 탭. 관리자에게만 보인다 (BR-K3). */
export function StaffAdminPage() {
  const [list, setList] = useState<ListKey>("staff");

  return (
    <div className={styles.page}>
      <Tabs
        label="직원 관리"
        items={LISTS}
        selected={list}
        onSelect={setList}
      />
      <div role="tabpanel" className={styles.content}>
        {list === "staff" && <StaffPage />}
        {list === "accessLogs" && <AccessLogList />}
      </div>
    </div>
  );
}
