import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import { restartApp } from "../../../api/appCommands";
import { startHandover } from "../../gate/handover/api";
import { useIsHostPc } from "../../gate/run-mode/useRunMode";
import {
  approveDevice,
  hostAddress,
  listDeviceRequests,
  listDevices,
  rejectDevice,
  revokeDevice,
} from "../devices/api";
import { DevicePanel } from "../devices/DevicePanel";
import { useOpenRequestCount } from "../devices/useOpenRequestCount";
import {
  createBackup,
  listBackups,
  prepareRestore,
  previewRestore,
  setSecondLocation,
} from "./api";
import { BackupPanel } from "./BackupPanel";

vi.mock("../../../api/appCommands", () => ({
  pickBackupFile: vi.fn(),
  restartApp: vi.fn(),
}));

vi.mock("./api", () => ({
  listBackups: vi.fn(),
  createBackup: vi.fn(),
  previewRestore: vi.fn(),
  prepareRestore: vi.fn(),
  setSecondLocation: vi.fn(),
}));

vi.mock("../../gate/handover/api", () => ({
  startHandover: vi.fn(),
}));

vi.mock("../../gate/run-mode/useRunMode", () => ({
  useIsHostPc: vi.fn(),
}));

vi.mock("../devices/api", () => ({
  hostAddress: vi.fn(),
  listDeviceRequests: vi.fn(),
  approveDevice: vi.fn(),
  rejectDevice: vi.fn(),
  listDevices: vi.fn(),
  revokeDevice: vi.fn(),
}));

vi.mock("../devices/useOpenRequestCount", () => ({
  useOpenRequestCount: vi.fn(),
}));

function renderWithQuery(ui: React.ReactElement) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(restartApp).mockResolvedValue(undefined);
  vi.mocked(listBackups).mockResolvedValue({
    items: [
      {
        fileName: "manual-2026-06-01.db",
        takenAt: "2026-06-01T10:00:00",
        kind: "manual",
        courseId: null,
        courseName: null,
        size: 1024 * 64,
      },
    ],
    lastFailure: null,
    lastRestore: null,
    secondLocation: {
      path: null,
      reachable: false,
      lastCopiedAt: null,
      changeable: true,
    },
  });
  vi.mocked(createBackup).mockResolvedValue({
    fileName: "manual-2026-06-02.db",
    takenAt: "2026-06-02T11:00:00",
    kind: "manual",
    courseId: null,
    courseName: null,
    size: 1024 * 64,
  });
  vi.mocked(previewRestore).mockResolvedValue({
    takenAt: "2026-06-01T10:00:00",
    kind: "manual",
    termNames: ["2026년 2학기"],
    memberCount: 12,
    applicationCount: 30,
  });
  vi.mocked(prepareRestore).mockResolvedValue(undefined);
  vi.mocked(setSecondLocation).mockResolvedValue({
    path: null,
    reachable: false,
    lastCopiedAt: null,
    changeable: true,
  });
  vi.mocked(startHandover).mockResolvedValue(undefined);

  vi.mocked(useIsHostPc).mockReturnValue(true);
  vi.mocked(useOpenRequestCount).mockReturnValue(1);
  vi.mocked(hostAddress).mockResolvedValue("192.168.0.10:41234");
  vi.mocked(listDeviceRequests).mockResolvedValue({
    items: [
      {
        id: 5,
        name: "접수 노트북",
        code: "123456",
        requestedAt: "2026-06-01T01:00:00Z",
      },
    ],
  });
  vi.mocked(approveDevice).mockResolvedValue(undefined);
  vi.mocked(rejectDevice).mockResolvedValue(undefined);
  vi.mocked(listDevices).mockResolvedValue({
    items: [
      {
        id: 8,
        name: "상담실 PC",
        approvedAt: "2026-05-20T01:00:00Z",
        approvedBy: "관리자1",
      },
    ],
  });
  vi.mocked(revokeDevice).mockResolvedValue(undefined);
});

test("백업 탭에서 지금 백업, 복원 확인 대화상자, 호스트 넘기기 PIN 확인이 동작한다", async () => {
  const user = userEvent.setup();
  renderWithQuery(<BackupPanel />);

  expect(await screen.findByText("64.0 KB")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "지금 백업" }));
  expect(createBackup).toHaveBeenCalledTimes(1);
  expect(await screen.findByRole("status")).toHaveTextContent("백업했습니다");

  await user.click(screen.getByRole("button", { name: /백업으로 복원/ }));
  const restoreDialog = await screen.findByRole("dialog", {
    name: "이 백업으로 되돌릴까요?",
  });
  expect(
    within(restoreDialog).getByRole("button", { name: "그만두기" }),
  ).toHaveFocus();
  await user.click(
    within(restoreDialog).getByRole("button", { name: "그만두기" }),
  );

  await user.click(screen.getByRole("button", { name: "호스트 넘기기" }));
  const handoverDialog = screen.getByRole("dialog", {
    name: "이 PC의 호스트를 다른 PC로 넘길까요?",
  });
  await user.click(
    within(handoverDialog).getByRole("button", { name: "넘기기 시작" }),
  );
  expect(startHandover).not.toHaveBeenCalled();
  expect(
    within(handoverDialog).getByText("PIN을 넣어 주세요"),
  ).toBeInTheDocument();

  await user.type(within(handoverDialog).getByLabelText("내 PIN"), "123456");
  await user.click(
    within(handoverDialog).getByRole("button", { name: "넘기기 시작" }),
  );
  expect(startHandover).toHaveBeenCalledWith("123456");
  expect(restartApp).toHaveBeenCalledTimes(1);
});

test("기기 탭에서 승인 요청 허락과 승인한 기기 취소 확인 대화상자가 동작한다", async () => {
  const user = userEvent.setup();
  renderWithQuery(<DevicePanel />);

  expect(await screen.findByText("192.168.0.10:41234")).toBeInTheDocument();
  expect(screen.getByText("123456")).toBeInTheDocument();

  await user.type(screen.getByLabelText("관리자 PIN"), "123456");
  await user.click(screen.getByRole("button", { name: "허락" }));
  expect(approveDevice).toHaveBeenCalledWith(5, "123456");
  expect(await screen.findByRole("status")).toHaveTextContent(
    "접수 노트북을(를) 허락했습니다",
  );

  await user.click(screen.getByRole("button", { name: "승인 취소" }));
  const confirm = screen.getByRole("dialog", {
    name: "상담실 PC의 승인을 취소할까요?",
  });
  expect(
    within(confirm).getByRole("button", { name: "그만두기" }),
  ).toHaveFocus();
  await user.click(within(confirm).getByRole("button", { name: "승인 취소" }));

  expect(revokeDevice).toHaveBeenCalledWith(8);
  expect(await screen.findByRole("status")).toHaveTextContent(
    "상담실 PC의 승인을 취소했습니다",
  );
});
