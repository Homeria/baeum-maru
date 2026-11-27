import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import type { Session } from "../api/session";
import { useIsHostPc } from "../features/gate/run-mode/useRunMode";
import { listBackups } from "../features/workspace/backups/api";
import { useChangeNotices } from "../features/workspace/changes/useChangeNotices";
import { useOpenRequestCount } from "../features/workspace/devices/useOpenRequestCount";
import { useHostVersion } from "../features/workspace/host/useHostVersion";
import { MainScreen } from "./MainScreen";

vi.mock("../features/gate/run-mode/useRunMode", () => ({
  useIsHostPc: vi.fn(),
}));

vi.mock("../features/workspace/changes/useChangeNotices", () => ({
  useChangeNotices: vi.fn(),
}));

vi.mock("../features/workspace/devices/useOpenRequestCount", () => ({
  useOpenRequestCount: vi.fn(),
}));

vi.mock("../features/workspace/host/useHostVersion", () => ({
  useHostVersion: vi.fn(),
}));

vi.mock("../features/workspace/backups/api", () => ({
  listBackups: vi.fn(),
}));

vi.mock("../features/workspace/members/MembersPage", () => ({
  MembersPage: () => <p>회원 화면</p>,
}));
vi.mock("../features/workspace/courses/CoursesPage", () => ({
  CoursesPage: () => <p>강좌 화면</p>,
}));
vi.mock("../features/workspace/staff/StaffAdminPage", () => ({
  StaffAdminPage: ({ activeTab }: { activeTab?: string }) => (
    <p>직원 관리 화면: {activeTab}</p>
  ),
}));

function renderMain(session: Session) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <MainScreen session={session} />
    </QueryClientProvider>,
  );
}

test("관리자 메뉴·연결 끊김·기기 승인 요청 하단 알림을 표시하고 확인하기로 기기 탭에 들어간다", async () => {
  const user = userEvent.setup();
  vi.mocked(useIsHostPc).mockReturnValue(true);
  vi.mocked(useChangeNotices).mockReturnValue("lost");
  vi.mocked(useOpenRequestCount).mockReturnValue(2);
  vi.mocked(useHostVersion).mockReturnValue({
    data: { hostVersion: "0.1.0", compatible: true },
    isPending: false,
    isError: false,
  } as ReturnType<typeof useHostVersion>);
  vi.mocked(listBackups).mockResolvedValue({
    items: [],
    lastFailure: null,
    lastRestore: null,
    secondLocation: {
      path: null,
      reachable: false,
      lastCopiedAt: null,
      changeable: false,
    },
  });

  renderMain({
    token: "tok",
    staff: { id: 1, name: "관리자1", role: "admin" },
    locked: false,
  });

  const nav = screen.getByRole("navigation", { name: "업무 메뉴" });
  expect(within(nav).getByText("수강 사무")).toBeInTheDocument();
  expect(within(nav).getByText("기관 운영")).toBeInTheDocument();
  expect(screen.getByText("호스트 연결됨")).toBeInTheDocument();

  expect(screen.getByRole("alert")).toHaveTextContent(
    "호스트와 연결이 끊겼습니다",
  );
  expect(screen.getByRole("status")).toHaveTextContent(
    "새 기기 승인 요청 2건이 있습니다",
  );

  await user.click(screen.getByRole("button", { name: "확인하기" }));
  expect(screen.getByText("직원 관리 화면: devices")).toBeInTheDocument();
});

test("조회 전용 역할에게는 직원 관리 메뉴를 숨기고 머리줄에 조회 전용 꼬리표를 띄운다", () => {
  vi.mocked(useIsHostPc).mockReturnValue(true);
  vi.mocked(useChangeNotices).mockReturnValue("open");
  vi.mocked(useOpenRequestCount).mockReturnValue(0);
  vi.mocked(useHostVersion).mockReturnValue({
    data: { hostVersion: "0.1.0", compatible: true },
    isPending: false,
    isError: false,
  } as ReturnType<typeof useHostVersion>);

  renderMain({
    token: "tok",
    staff: { id: 2, name: "조회직원", role: "read_only" },
    locked: false,
  });

  const nav = screen.getByRole("navigation", { name: "업무 메뉴" });
  expect(
    within(nav).queryByRole("button", { name: "직원 관리" }),
  ).not.toBeInTheDocument();
  expect(
    screen.getByText("지금 역할로는 보기만 할 수 있습니다"),
  ).toBeInTheDocument();
});
