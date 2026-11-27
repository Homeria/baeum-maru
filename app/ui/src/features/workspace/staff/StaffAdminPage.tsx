import { useState } from "react";
import { Tabs, type TabItem } from "../../../components/tabs/Tabs";
import { AccessLogList } from "../access-logs/AccessLogList";
import { BackupPanel } from "../backups/BackupPanel";
import { DevicePanel } from "../devices/DevicePanel";
import styles from "../reference-data/ReferenceDataPage.module.css";
import { StaffPage } from "./StaffPage";

/** 직원 관리 화면 안의 작은 탭 키. */
export type StaffAdminTab = "staff" | "accessLogs" | "backups" | "devices";

const LISTS: readonly TabItem<StaffAdminTab>[] = [
  { key: "staff", label: "직원" },
  { key: "accessLogs", label: "접속 기록" },
  { key: "backups", label: "백업" },
  { key: "devices", label: "기기" },
];

/** {@link StaffAdminPage}의 속성. */
export interface StaffAdminPageProps {
  /** 외부(예: 하단 알림 "확인하기")에서 특정 작은 탭을 열 때 넘긴다. */
  activeTab?: StaffAdminTab;
  /** 작은 탭이 바뀔 때 호출된다. */
  onTabChange?: (tab: StaffAdminTab) => void;
}

/** 직원 관리 탭: 직원, 접속 기록, 백업, 기기. 관리자에게만 보인다 (BR-K3). */
export function StaffAdminPage({
  activeTab,
  onTabChange,
}: StaffAdminPageProps) {
  const [internalTab, setInternalTab] = useState<StaffAdminTab>("staff");
  const list = activeTab ?? internalTab;

  const select = (next: StaffAdminTab) => {
    if (activeTab === undefined) {
      setInternalTab(next);
    }
    onTabChange?.(next);
  };

  return (
    <div className={styles.page}>
      <Tabs label="직원 관리" items={LISTS} selected={list} onSelect={select} />
      <div role="tabpanel" className={styles.content}>
        {list === "staff" && <StaffPage />}
        {list === "accessLogs" && <AccessLogList />}
        {list === "backups" && <BackupPanel />}
        {list === "devices" && <DevicePanel />}
      </div>
    </div>
  );
}
