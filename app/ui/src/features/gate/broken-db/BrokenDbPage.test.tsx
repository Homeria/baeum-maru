import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { pickBackupFile } from "../../../api/appCommands";
import type { StartupStateResponse } from "../../../api/generated/StartupStateResponse";
import { previewBrokenDbBackup, restoreBrokenDb } from "./api";
import { BrokenDbPage } from "./BrokenDbPage";
import { StartupProblemPage } from "./StartupProblemPage";

vi.mock("./api", () => ({
  previewBrokenDbBackup: vi.fn(),
  restoreBrokenDb: vi.fn(),
}));

vi.mock("../../../api/appCommands", () => ({
  pickBackupFile: vi.fn(),
}));

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

test("추천 백업이 있으면 미리보기를 보여 주고 되돌리기를 실행한다", async () => {
  const user = userEvent.setup();
  vi.mocked(restoreBrokenDb).mockResolvedValue(undefined);
  const startup: StartupStateResponse = {
    state: "broken",
    backups: [],
    suggestion: {
      fileName: "daily-20261126.db",
      preview: {
        takenAt: "2026-11-26T00:00:00Z",
        kind: "daily",
        termNames: ["2026년 4분기"],
        memberCount: 120,
        applicationCount: 340,
      },
    },
  };

  renderWithQuery(<BrokenDbPage startup={startup} />);

  expect(
    screen.getByRole("heading", {
      level: 3,
      name: "가장 최근 정상 백업으로 되돌릴까요?",
    }),
  ).toBeInTheDocument();
  expect(screen.getByText(/120명/)).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "되돌리기" }));
  expect(restoreBrokenDb).toHaveBeenCalledWith({
    fileName: "daily-20261126.db",
  });
});

test("추천 백업이 없으면 경고 배너를 보여 주고 외부 백업 파일 미리보기를 불러온다", async () => {
  const user = userEvent.setup();
  vi.mocked(pickBackupFile).mockResolvedValue("/usb/backup.db");
  vi.mocked(previewBrokenDbBackup).mockResolvedValue({
    takenAt: "2026-11-25T00:00:00Z",
    kind: "manual",
    termNames: ["2026년 4분기"],
    memberCount: 80,
    applicationCount: 200,
  });

  renderWithQuery(
    <BrokenDbPage
      startup={{ state: "broken", backups: [], suggestion: null }}
    />,
  );

  expect(screen.getByRole("alert")).toHaveTextContent(
    "쓸 수 있는 백업을 찾지 못했습니다",
  );

  await user.click(
    screen.getByRole("button", { name: "다른 곳의 백업 파일 고르기" }),
  );
  expect(
    await screen.findByRole("heading", {
      level: 3,
      name: "이 백업으로 되돌릴까요?",
    }),
  ).toBeInTheDocument();
  expect(previewBrokenDbBackup).toHaveBeenCalledWith({
    path: "/usb/backup.db",
  });
});

test("StartupProblemPage는 시작 불가 원인을 alert 배너로 보여 준다", () => {
  render(<StartupProblemPage state="newer_version" />);

  expect(
    screen.getByRole("heading", {
      level: 1,
      name: "앱을 새 버전으로 바꿔 주세요",
    }),
  ).toBeInTheDocument();
  expect(screen.getByRole("alert")).toHaveTextContent("더 새 버전");
});
