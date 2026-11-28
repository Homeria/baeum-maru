import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { Session } from "../api/session";
import {
  AppShell,
  type AppShellNavSection,
} from "../components/shell/AppShell";
import type { BottomAlertItem } from "../components/shell/BottomAlertStack";
import { StatusPill } from "../components/ui/StatusPill";
import { can, type Action } from "../features/gate/auth/roles";
import { SignedInStaff } from "../features/gate/auth/SignedInStaff";
import { ViewOnlyNotice } from "../features/gate/auth/ViewOnlyNotice";
import { useIsHostPc } from "../features/gate/run-mode/useRunMode";
import { ApplicationsPage } from "../features/workspace/applications/ApplicationsPage";
import { listBackups } from "../features/workspace/backups/api";
import { BACKUP_KIND_LABELS } from "../features/workspace/backups/backupData";
import { useChangeNotices } from "../features/workspace/changes/useChangeNotices";
import { CoursesPage } from "../features/workspace/courses/CoursesPage";
import { useOpenRequestCount } from "../features/workspace/devices/useOpenRequestCount";
import { HostStatus } from "../features/workspace/host/HostStatus";
import { MembersPage } from "../features/workspace/members/MembersPage";
import { ReferenceDataPage } from "../features/workspace/reference-data/ReferenceDataPage";
import { SelectionResultsPage } from "../features/workspace/selection-results/SelectionResultsPage";
import { SelectionPage } from "../features/workspace/selection/SelectionPage";
import {
  StaffAdminPage,
  type StaffAdminTab,
} from "../features/workspace/staff/StaffAdminPage";

type MenuKey =
  | "members"
  | "courses"
  | "applications"
  | "selection"
  | "selectionResults"
  | "referenceData"
  | "staff";

const MENU_TITLES: Record<MenuKey, string> = {
  members: "회원",
  courses: "강좌",
  applications: "신청",
  selection: "추첨",
  selectionResults: "선발 결과",
  referenceData: "기초 정보",
  staff: "직원 관리",
};

/** 탭별 주 편집 권한. 권한이 없으면 머리줄에 조회 전용 꼬리표를 띄운다. */
const MENU_EDIT_ACTION: Partial<Record<MenuKey, Action>> = {
  members: "editMembers",
  courses: "editCourses",
  applications: "takeApplications",
  selection: "runLottery",
  selectionResults: "recordWithdrawals",
  referenceData: "editReferenceData",
};

/** {@link MainScreen}의 속성. */
export interface MainScreenProps {
  session: Session;
}

/** 로그인한 뒤의 메인 화면: 왼쪽 남색 메뉴, 작업 영역, 하단 고정 알림 띠. */
export function MainScreen({ session }: MainScreenProps) {
  const [menu, setMenu] = useState<MenuKey>("members");
  const [staffTab, setStaffTab] = useState<StaffAdminTab>("staff");
  const [dismissedRequestCount, setDismissedRequestCount] = useState<
    number | null
  >(null);
  const [dismissedBackupAt, setDismissedBackupAt] = useState<string | null>(
    null,
  );

  const role = session.staff.role;
  const manageStaff = can(role, "manageStaff");
  const manageBackups = can(role, "manageBackups");
  const host = useIsHostPc();
  // BR-K15: 새 기기 승인은 관리자가 호스트 PC에서 한다
  const approvesDevices = host && can(role, "manageDevices") && !session.locked;
  // ADR 0008: 다른 PC가 바꾼 내용을 바로 다시 받는다. 잠긴 동안에는 받지 않는다
  const connection = useChangeNotices(!session.locked);
  const openRequests = useOpenRequestCount(approvesDevices);

  const backups = useQuery({
    queryKey: ["backups"],
    queryFn: listBackups,
    enabled: manageBackups && !session.locked,
    staleTime: 60 * 1000,
  });

  const sections: readonly AppShellNavSection<MenuKey>[] = [
    {
      label: "수강 사무",
      items: [
        { key: "members", label: "회원" },
        { key: "courses", label: "강좌" },
        { key: "applications", label: "신청" },
        { key: "selection", label: "추첨" },
        { key: "selectionResults", label: "선발 결과" },
      ],
    },
    {
      label: "기관 운영",
      items: manageStaff
        ? [
            { key: "referenceData", label: "기초 정보" },
            {
              key: "staff",
              label: "직원 관리",
              badge:
                openRequests > 0 ? (
                  <StatusPill tone="primary">{openRequests}</StatusPill>
                ) : undefined,
            },
          ]
        : [{ key: "referenceData", label: "기초 정보" }],
    },
  ];

  const alerts: BottomAlertItem[] = [];
  if (connection === "lost") {
    alerts.push({
      id: "connection-lost",
      tone: "danger",
      message:
        "호스트와 연결이 끊겼습니다. 다시 연결하는 중입니다. 그동안 다른 PC가 바꾼 내용은 화면에 보이지 않습니다.",
    });
  }
  if (
    approvesDevices &&
    openRequests > 0 &&
    dismissedRequestCount !== openRequests
  ) {
    alerts.push({
      id: "device-requests",
      tone: "info",
      message: `승인을 기다리는 기기가 ${openRequests}대 있습니다.`,
      actionLabel: "확인하기",
      onAction: () => {
        setMenu("staff");
        setStaffTab("devices");
      },
      onDismiss: () => setDismissedRequestCount(openRequests),
    });
  }
  const lastFailure = backups.data?.lastFailure ?? null;
  if (
    manageBackups &&
    !session.locked &&
    lastFailure !== null &&
    dismissedBackupAt !== lastFailure.at
  ) {
    alerts.push({
      id: "backup-failure",
      tone: "danger",
      message: `최근 ${BACKUP_KIND_LABELS[lastFailure.kind]}에 실패했습니다. 백업 상태를 확인해 주세요.`,
      actionLabel: "확인하기",
      onAction: () => {
        setMenu("staff");
        setStaffTab("backups");
      },
      onDismiss: () => setDismissedBackupAt(lastFailure.at),
    });
  }

  const editAction = MENU_EDIT_ACTION[menu];
  const isViewOnly = editAction !== undefined && !can(role, editAction);

  return (
    <AppShell
      sections={sections}
      selected={menu}
      onSelect={setMenu}
      pageTitle={MENU_TITLES[menu]}
      headerStatus={isViewOnly ? <ViewOnlyNotice /> : undefined}
      hostStatus={<HostStatus />}
      sidebarFooter={<SignedInStaff session={session} />}
      alerts={alerts}
      locked={session.locked}
    >
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
        <StaffAdminPage activeTab={staffTab} onTabChange={setStaffTab} />
      ) : null}
    </AppShell>
  );
}
