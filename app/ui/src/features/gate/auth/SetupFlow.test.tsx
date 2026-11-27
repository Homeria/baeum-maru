import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import { listLoginStaff, login } from "./api";
import { RecoveryCodeNotice } from "./RecoveryCodeNotice";
import { setup } from "./setupApi";
import { SetupPage } from "./SetupPage";

vi.mock("./setupApi", () => ({
  setup: vi.fn(),
}));

vi.mock("./api", () => ({
  listLoginStaff: vi.fn(),
  login: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
});

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

test("SetupPage는 3단계를 보여 주고 PIN 확인이 다르면 오류와 함께 초점을 옮긴다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();

  renderWithQuery(<SetupPage onDone={onDone} />);

  const progress = screen.getByRole("navigation", { name: "진행 단계" });
  expect(within(progress).getAllByRole("listitem")).toHaveLength(3);
  expect(within(progress).getByText("기초 정보 미리 넣기")).toBeInTheDocument();

  await user.type(screen.getByLabelText("관리자 이름"), "관리자1");
  await user.type(screen.getByLabelText("PIN"), "123456");
  await user.type(screen.getByLabelText("PIN 한 번 더"), "654321");
  await user.click(screen.getByRole("button", { name: "관리자 만들기" }));

  const confirmInput = screen.getByLabelText("PIN 한 번 더");
  expect(confirmInput).toHaveAttribute("aria-invalid", "true");
  expect(confirmInput).toHaveFocus();
  expect(setup).not.toHaveBeenCalled();
});

test("SetupPage는 관리자 생성 뒤 자동 로그인까지 마치면 autoSignedIn=true로 넘긴다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();
  vi.mocked(setup).mockResolvedValue({
    recoveryCode: "K7QF-M2XA-9TRE-HW4C",
  });
  vi.mocked(listLoginStaff).mockResolvedValue({
    items: [{ id: 1, name: "관리자1", role: "admin" }],
  });
  vi.mocked(login).mockResolvedValue({
    token: "tok",
    staff: { id: 1, name: "관리자1", role: "admin" },
  });

  renderWithQuery(<SetupPage onDone={onDone} />);

  await user.type(screen.getByLabelText("관리자 이름"), "관리자1");
  await user.type(screen.getByLabelText("PIN"), "123456");
  await user.type(screen.getByLabelText("PIN 한 번 더"), "123456");
  await user.click(screen.getByRole("button", { name: "관리자 만들기" }));

  expect(login).toHaveBeenCalledWith({ staffId: 1, pin: "123456" });
  expect(onDone).toHaveBeenCalledWith({
    recoveryCode: "K7QF-M2XA-9TRE-HW4C",
    autoSignedIn: true,
  });
});

test("SetupPage는 자동 로그인만 실패해도 복구 코드를 잃지 않고 autoSignedIn=false로 넘긴다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();
  vi.mocked(setup).mockResolvedValue({
    recoveryCode: "K7QF-M2XA-9TRE-HW4C",
  });
  vi.mocked(listLoginStaff).mockRejectedValue(new Error("네트워크 오류"));

  renderWithQuery(<SetupPage onDone={onDone} />);

  await user.type(screen.getByLabelText("관리자 이름"), "관리자1");
  await user.type(screen.getByLabelText("PIN"), "123456");
  await user.type(screen.getByLabelText("PIN 한 번 더"), "123456");
  await user.click(screen.getByRole("button", { name: "관리자 만들기" }));

  expect(onDone).toHaveBeenCalledWith({
    recoveryCode: "K7QF-M2XA-9TRE-HW4C",
    autoSignedIn: false,
  });
});

test("RecoveryCodeNotice는 처음 설정에서는 3단계, PIN 복구에서는 2단계를 보여 주고 체크 후 다음을 연다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();
  const printSpy = vi.spyOn(window, "print").mockImplementation(() => {});

  const { rerender } = render(
    <RecoveryCodeNotice code="K7QF-M2XA-9TRE-HW4C" onDone={onDone} />,
  );

  expect(
    within(screen.getByRole("navigation", { name: "진행 단계" })).getAllByRole(
      "listitem",
    ),
  ).toHaveLength(3);
  expect(screen.getByText("K7QF-M2XA-9TRE-HW4C")).toBeInTheDocument();
  const nextButton = screen.getByRole("button", { name: "다음" });
  expect(nextButton).toBeDisabled();

  await user.click(screen.getByRole("button", { name: "인쇄" }));
  expect(printSpy).toHaveBeenCalledTimes(1);

  await user.click(
    screen.getByRole("checkbox", { name: "인쇄하거나 적어 두었습니다" }),
  );
  expect(nextButton).toBeEnabled();

  await user.click(nextButton);
  expect(onDone).toHaveBeenCalledTimes(1);

  rerender(
    <RecoveryCodeNotice
      code="ABCD-1234-EFGH-5678"
      mode="recovery"
      onDone={onDone}
    />,
  );
  expect(
    within(screen.getByRole("navigation", { name: "진행 단계" })).getAllByRole(
      "listitem",
    ),
  ).toHaveLength(2);
  expect(screen.queryByText("기초 정보 미리 넣기")).not.toBeInTheDocument();

  printSpy.mockRestore();
});
