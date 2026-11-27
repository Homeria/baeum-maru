import { useState } from "react";
import { Tabs, type TabItem } from "../components/tabs/Tabs";
import { ApplicationsPage } from "../features/workspace/applications/ApplicationsPage";
import { CoursesPage } from "../features/workspace/courses/CoursesPage";
import type { Session } from "../api/session";
import { SignedInStaff } from "../features/gate/auth/SignedInStaff";
import { ConnectionLostBar } from "../features/changes/ConnectionLostBar";
import { useChangeNotices } from "../features/changes/useChangeNotices";
import { DeviceRequestNotice } from "../features/devices/DeviceRequestNotice";
import { HostStatus } from "../features/host/HostStatus";
import { MembersPage } from "../features/workspace/members/MembersPage";
import { ReferenceDataPage } from "../features/reference-data/ReferenceDataPage";
import { SelectionPage } from "../features/workspace/selection/SelectionPage";
import { SelectionResultsPage } from "../features/workspace/selection-results/SelectionResultsPage";
import { can } from "../features/gate/auth/roles";
import { useIsHostPc } from "../features/gate/run-mode/useRunMode";
import { StaffAdminPage } from "../features/staff/StaffAdminPage";
import styles from "./App.module.css";

type MenuKey =
  | "members"
  | "courses"
  | "applications"
  | "selection"
  | "selectionResults"
  | "referenceData"
  | "staff";

const MENU: readonly TabItem<MenuKey>[] = [
  { key: "members", label: "회원" },
  { key: "courses", label: "강좌" },
  { key: "applications", label: "신청" },
  { key: "selection", label: "추첨" },
  { key: "selectionResults", label: "선발 결과" },
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
  const host = useIsHostPc();
  // BR-K15: 새 기기 승인은 관리자가 호스트 PC에서 한다
  const approvesDevices =
    host && can(session.staff.role, "manageDevices") && !session.locked;
  // ADR 0008: 다른 PC가 바꾼 내용을 바로 다시 받는다. 잠긴 동안에는 받지 않는다
  const connection = useChangeNotices(!session.locked);

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
      <ConnectionLostBar lost={connection === "lost"} />
      <DeviceRequestNotice active={approvesDevices} />
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
        ) : menu === "selectionResults" ? (
          <SelectionResultsPage />
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
