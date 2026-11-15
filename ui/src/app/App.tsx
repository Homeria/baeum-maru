import { useState } from "react";
import { Tabs, type TabItem } from "../components/Tabs/Tabs";
import { HostStatus } from "../features/host/HostStatus";
import { MembersPage } from "../features/members/MembersPage";
import { ReferenceDataPage } from "../features/reference-data/ReferenceDataPage";
import styles from "./App.module.css";

type MenuKey =
  "members" | "courses" | "applications" | "selection" | "referenceData";

const MENU: readonly TabItem<MenuKey>[] = [
  { key: "members", label: "회원" },
  { key: "courses", label: "강좌" },
  { key: "applications", label: "신청" },
  { key: "selection", label: "선발" },
  { key: "referenceData", label: "기초 정보" },
];

export function App() {
  const [menu, setMenu] = useState<MenuKey>("members");

  return (
    <div className={styles.app}>
      <header className={styles.header}>
        <h1 className={styles.title}>배움마루</h1>
        <HostStatus />
      </header>
      <nav className={styles.nav}>
        <Tabs items={MENU} selected={menu} onSelect={setMenu} />
      </nav>
      <main role="tabpanel" className={styles.main}>
        {menu === "members" ? (
          <MembersPage />
        ) : menu === "referenceData" ? (
          <ReferenceDataPage />
        ) : (
          "준비 중입니다."
        )}
      </main>
    </div>
  );
}
