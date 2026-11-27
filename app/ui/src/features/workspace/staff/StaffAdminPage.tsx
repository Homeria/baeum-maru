import { useState } from "react";
import { Tabs, type TabItem } from "../../../components/tabs/Tabs";
import { AccessLogList } from "../access-logs/AccessLogList";
import { BackupPanel } from "../backups/BackupPanel";
import { DevicePanel } from "../devices/DevicePanel";
import styles from "../reference-data/ReferenceDataPage.module.css";
import { StaffPage } from "./StaffPage";

type ListKey = "staff" | "accessLogs" | "backups" | "devices";

const LISTS: readonly TabItem<ListKey>[] = [
  { key: "staff", label: "직원" },
  { key: "accessLogs", label: "접속 기록" },
  { key: "backups", label: "백업" },
  { key: "devices", label: "기기" },
];

/** 직원 관리 탭: 직원, 접속 기록, 백업, 기기. 관리자에게만 보인다 (BR-K3). */
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
        {list === "backups" && <BackupPanel />}
        {list === "devices" && <DevicePanel />}
      </div>
    </div>
  );
}
