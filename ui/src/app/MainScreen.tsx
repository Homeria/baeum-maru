import { useState } from "react";
import { Tabs, type TabItem } from "../components/Tabs/Tabs";
import { CoursesPage } from "../features/courses/CoursesPage";
import type { Session } from "../api/session";
import { SignedInStaff } from "../features/auth/SignedInStaff";
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

/** {@link MainScreen}의 속성. */
export interface MainScreenProps {
  session: Session;
}

/** 로그인한 뒤의 화면: 위쪽 메뉴와 고른 메뉴의 내용. */
export function MainScreen({ session }: MainScreenProps) {
  const [menu, setMenu] = useState<MenuKey>("members");

  return (
    // 잠긴 동안에는 눌리거나 읽히지 않게 (잠금 화면이 위를 덮는다)
    <div className={styles.app} inert={session.locked}>
      <header className={styles.header}>
        <h1 className={styles.title}>배움마루</h1>
        <div className={styles.headerRight}>
          <HostStatus />
          <SignedInStaff session={session} />
        </div>
      </header>
      <nav className={styles.nav}>
        <Tabs items={MENU} selected={menu} onSelect={setMenu} />
      </nav>
      <main role="tabpanel" className={styles.main}>
        {menu === "members" ? (
          <MembersPage />
        ) : menu === "courses" ? (
          <CoursesPage />
        ) : menu === "referenceData" ? (
          <ReferenceDataPage />
        ) : (
          "준비 중입니다."
        )}
      </main>
    </div>
  );
}
