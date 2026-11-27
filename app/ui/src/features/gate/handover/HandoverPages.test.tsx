import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { changeRunMode } from "../../../api/runMode";
import {
  abortHandover,
  cancelHandover,
  handedOverAdmins,
  handoverFile,
  pickHandoverFile,
  previewTakeOver,
  saveHandoverFile,
  takeOver,
} from "./api";
import { HandedOverPage } from "./HandedOverPage";
import { HandingOverPage } from "./HandingOverPage";
import { TakeOverPage } from "./TakeOverPage";

vi.mock("./api", () => ({
  handoverFile: vi.fn(),
  saveHandoverFile: vi.fn(),
  abortHandover: vi.fn(),
  handedOverAdmins: vi.fn(),
  cancelHandover: vi.fn(),
  pickHandoverFile: vi.fn(),
  previewTakeOver: vi.fn(),
  takeOver: vi.fn(),
}));

vi.mock("../../../api/runMode", () => ({
  changeRunMode: vi.fn(),
}));

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

test("HandingOverPage는 넘기기 파일 내용을 보여 주고 미저장 안내와 돌아가기를 처리한다", async () => {
  const user = userEvent.setup();
  vi.mocked(handoverFile).mockResolvedValue({
    termNames: ["2026년 4분기"],
    memberCount: 50,
    applicationCount: 120,
  });
  vi.mocked(saveHandoverFile).mockResolvedValue(false);
  vi.mocked(abortHandover).mockResolvedValue(undefined);

  renderWithQuery(<HandingOverPage />);

  expect(await screen.findByText(/회원 50명/)).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "넘기기 파일 저장" }));
  expect(await screen.findByRole("status")).toHaveTextContent(
    "저장하지 않았습니다",
  );

  await user.click(
    screen.getByRole("button", { name: "넘기지 않고 돌아가기" }),
  );
  expect(abortHandover).toHaveBeenCalledTimes(1);
});

test("HandedOverPage는 넘기기 취소 대화상자와 접속 모드 전환 확인 창을 연다", async () => {
  const user = userEvent.setup();
  vi.mocked(handedOverAdmins).mockResolvedValue([{ id: 1, name: "관리자1" }]);
  vi.mocked(cancelHandover).mockResolvedValue(undefined);
  vi.mocked(changeRunMode).mockResolvedValue(undefined);

  renderWithQuery(<HandedOverPage />);

  await user.click(screen.getByRole("button", { name: "넘기기 취소" }));
  expect(await screen.findByText("관리자1")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "취소하고 다시 켜기" }));
  expect(screen.getByLabelText("PIN")).toHaveAttribute("aria-invalid", "true");
  await user.click(screen.getByRole("button", { name: "그만두기" }));

  await user.click(
    screen.getByRole("button", { name: "다른 PC에 접속하는 모드로 바꾸기" }),
  );
  expect(
    screen.getByRole("dialog", {
      name: "이 PC를 다른 PC에 접속하는 모드로 바꿀까요?",
    }),
  ).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "바꾸고 다시 켜기" }));
  expect(changeRunMode).toHaveBeenCalledWith("remote");
});

test("TakeOverPage는 파일을 고르면 미리보기와 관리자 PIN을 보여 주고 넘겨받기를 실행한다", async () => {
  const user = userEvent.setup();
  vi.mocked(pickHandoverFile).mockResolvedValue("/usb/handover.bmaru");
  vi.mocked(previewTakeOver).mockResolvedValue({
    termNames: ["2026년 4분기"],
    memberCount: 50,
    applicationCount: 120,
    admins: [{ id: 1, name: "관리자1" }],
  });
  vi.mocked(takeOver).mockResolvedValue(undefined);

  renderWithQuery(<TakeOverPage onBack={() => {}} />);

  await user.click(screen.getByRole("button", { name: "넘기기 파일 고르기" }));
  expect(await screen.findByText("관리자1")).toBeInTheDocument();

  await user.click(screen.getByText("관리자1"));
  await user.type(screen.getByLabelText("PIN"), "123456");
  await user.click(
    screen.getByRole("button", { name: "이 파일로 호스트 시작" }),
  );
  expect(takeOver).toHaveBeenCalledWith("/usb/handover.bmaru", 1, "123456");
});
