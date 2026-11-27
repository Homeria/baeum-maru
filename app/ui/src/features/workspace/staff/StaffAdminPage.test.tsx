import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import type { StaffResponse } from "../../../api/generated/StaffResponse";
import { listAccessLogs } from "../access-logs/api";
import { createStaff, listStaff, resetStaffPin, updateStaff } from "./api";
import { StaffAdminPage } from "./StaffAdminPage";

vi.mock("./api", () => ({
  listStaff: vi.fn(),
  createStaff: vi.fn(),
  updateStaff: vi.fn(),
  resetStaffPin: vi.fn(),
}));

vi.mock("../access-logs/api", () => ({
  listAccessLogs: vi.fn(),
}));

vi.mock("../backups/BackupPanel", () => ({
  BackupPanel: () => <p>백업 화면</p>,
}));

vi.mock("../devices/DevicePanel", () => ({
  DevicePanel: () => <p>기기 화면</p>,
}));

const SAMPLE_STAFF: StaffResponse[] = [
  {
    id: 1,
    name: "관리자1",
    role: "admin",
    validUntil: null,
    disabled: false,
    locked: false,
  },
  {
    id: 2,
    name: "잠긴직원",
    role: "staff",
    validUntil: null,
    disabled: false,
    locked: true,
  },
  {
    id: 3,
    name: "퇴사직원",
    role: "temp_staff",
    validUntil: "2026-12-31",
    disabled: true,
    locked: false,
  },
];

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <StaffAdminPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(listStaff).mockResolvedValue({ items: SAMPLE_STAFF });
  vi.mocked(createStaff).mockResolvedValue({ id: 4 });
  vi.mocked(updateStaff).mockResolvedValue(SAMPLE_STAFF[0]);
  vi.mocked(resetStaffPin).mockResolvedValue(undefined);
  vi.mocked(listAccessLogs).mockResolvedValue({
    items: [
      {
        id: 10,
        occurredAt: "2026-06-01T01:02:03Z",
        staffId: 2,
        staffName: "잠긴직원",
        deviceName: null,
        event: "login_failed",
      },
    ],
    total: 1,
    page: 1,
    pageSize: 50,
  });
});

test("직원 목록 상태 꼬리표를 표시하고 직원 등록 서랍과 닫기 확인이 동작한다", async () => {
  const user = userEvent.setup();
  renderPage();

  expect(await screen.findByText("사용 중")).toBeInTheDocument();
  expect(screen.getByText("잠김")).toBeInTheDocument();
  expect(screen.getByText("사용 중지")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "직원 등록" }));
  const drawer = screen.getByRole("dialog", { name: "새 직원 등록" });

  await user.type(within(drawer).getByLabelText("이름"), "새직원");
  await user.type(within(drawer).getByLabelText("PIN"), "1234");
  await user.type(within(drawer).getByLabelText("PIN 한 번 더"), "1234");

  await user.click(within(drawer).getByRole("button", { name: "그만두기" }));
  expect(
    screen.getByRole("dialog", { name: "저장하지 않은 내용이 있습니다" }),
  ).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "계속 편집" }));

  await user.click(within(drawer).getByRole("button", { name: "등록" }));
  expect(createStaff).toHaveBeenCalledWith({
    name: "새직원",
    role: "staff",
    validUntil: null,
    pin: "1234",
  });
  expect(await screen.findByRole("status")).toHaveTextContent(
    "등록했습니다: 새직원",
  );
});

test("PIN 새로 정하기 모달에서 확인 불일치를 알리고 저장하면 알림을 띄운다", async () => {
  const user = userEvent.setup();
  renderPage();

  expect(await screen.findByText("잠긴직원")).toBeInTheDocument();
  const resetButtons = screen.getAllByRole("button", {
    name: "PIN 새로 정하기",
  });
  await user.click(resetButtons[1]);

  const dialog = screen.getByRole("dialog", { name: "PIN 새로 정하기" });
  expect(within(dialog).getByLabelText("새 PIN")).toHaveFocus();

  await user.type(within(dialog).getByLabelText("새 PIN"), "5678");
  await user.type(within(dialog).getByLabelText("새 PIN 한 번 더"), "0000");
  await user.click(
    within(dialog).getByRole("button", { name: "PIN 새로 정하기" }),
  );

  expect(resetStaffPin).not.toHaveBeenCalled();
  expect(within(dialog).getByRole("alert")).toBeInTheDocument();

  await user.clear(within(dialog).getByLabelText("새 PIN 한 번 더"));
  await user.type(within(dialog).getByLabelText("새 PIN 한 번 더"), "5678");
  await user.click(
    within(dialog).getByRole("button", { name: "PIN 새로 정하기" }),
  );

  expect(resetStaffPin).toHaveBeenCalledWith(2, "5678");
  expect(await screen.findByRole("status")).toHaveTextContent(
    "잠긴직원의 PIN을 새로 정했습니다",
  );
});

test("접속 기록 하위 탭에서 표와 실패 꼬리표를 보여 주고 새로 보기가 동작한다", async () => {
  const user = userEvent.setup();
  renderPage();

  await user.click(screen.getByRole("tab", { name: "접속 기록" }));
  expect(await screen.findByText("로그인 실패")).toBeInTheDocument();
  expect(screen.getByText("호스트 PC")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "새로 보기" }));
  expect(listAccessLogs).toHaveBeenCalledTimes(2);
});
