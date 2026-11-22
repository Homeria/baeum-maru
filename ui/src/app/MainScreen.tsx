import { useState } from "react";
import { Tabs, type TabItem } from "../components/Tabs/Tabs";
import { ApplicationsPage } from "../features/applications/ApplicationsPage";
import { CoursesPage } from "../features/courses/CoursesPage";
import type { Session } from "../api/session";
import { SignedInStaff } from "../features/auth/SignedInStaff";
import { HostStatus } from "../features/host/HostStatus";
import { MembersPage } from "../features/members/MembersPage";
import { ReferenceDataPage } from "../features/reference-data/ReferenceDataPage";
import { SelectionPage } from "../features/selection/SelectionPage";
import { can } from "../features/auth/roles";
import { StaffAdminPage } from "../features/staff/StaffAdminPage";
import styles from "./App.module.css";

type MenuKey =
  | "members"
  | "courses"
  | "applications"
  | "selection"
  | "referenceData"
  | "staff";

const MENU: readonly TabItem<MenuKey>[] = [
  { key: "members", label: "회원" },
  { key: "courses", label: "강좌" },
  { key: "applications", label: "신청" },
  { key: "selection", label: "추첨" },
  { key: "referenceData", label: "기초 정보" },
];

/** 직원 관리를 할 수 있는 역할(관리자)에게만 더 보이는 메뉴 (BR-K3). */
const ADMIN_MENU: readonly TabItem<MenuKey>[] = [
  ...MENU,
  { key: "staff", label: "직원 관리" },
];

/** {@link MainScreen}의 속성. */
export interface MainScreenProps {
  session: Session;
}

/** 로그인한 뒤의 화면: 위쪽 메뉴와 고른 메뉴의 내용. */
export function MainScreen({ session }: MainScreenProps) {
  const [menu, setMenu] = useState<MenuKey>("members");
  const manageStaff = can(session.staff.role, "manageStaff");

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
        <Tabs
          items={manageStaff ? ADMIN_MENU : MENU}
          selected={menu}
          onSelect={setMenu}
        />
      </nav>
      <main role="tabpanel" className={styles.main}>
        {menu === "members" ? (
          <MembersPage />
        ) : menu === "courses" ? (
          <CoursesPage />
        ) : menu === "applications" ? (
          <ApplicationsPage />
        ) : menu === "selection" ? (
          <SelectionPage />
        ) : menu === "referenceData" ? (
          <ReferenceDataPage />
        ) : menu === "staff" && manageStaff ? (
          <StaffAdminPage />
        ) : (
          "준비 중입니다."
        )}
      </main>
    </div>
  );
}
