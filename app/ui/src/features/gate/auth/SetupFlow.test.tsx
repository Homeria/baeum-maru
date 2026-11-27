import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { RecoveryCodeNotice } from "./RecoveryCodeNotice";
import { setup } from "./setupApi";
import { SetupPage } from "./SetupPage";

vi.mock("./setupApi", () => ({
  setup: vi.fn(),
}));

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

test("SetupPage는 PIN 확인이 다르면 오류를 보여 주고 초점을 옮긴다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();

  renderWithQuery(<SetupPage onDone={onDone} />);

  await user.type(screen.getByLabelText("관리자 이름"), "관리자1");
  await user.type(screen.getByLabelText("PIN"), "123456");
  await user.type(screen.getByLabelText("PIN 한 번 더"), "654321");
  await user.click(screen.getByRole("button", { name: "관리자 만들기" }));

  const confirmInput = screen.getByLabelText("PIN 한 번 더");
  expect(confirmInput).toHaveAttribute("aria-invalid", "true");
  expect(confirmInput).toHaveFocus();
  expect(setup).not.toHaveBeenCalled();
});

test("SetupPage는 관리자 생성에 성공하면 복구 코드를 onDone으로 넘긴다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();
  vi.mocked(setup).mockResolvedValue({
    recoveryCode: "K7QF-M2XA-9TRE-HW4C",
  });

  renderWithQuery(<SetupPage onDone={onDone} />);

  await user.type(screen.getByLabelText("관리자 이름"), "관리자1");
  await user.type(screen.getByLabelText("PIN"), "123456");
  await user.type(screen.getByLabelText("PIN 한 번 더"), "123456");
  await user.click(screen.getByRole("button", { name: "관리자 만들기" }));

  expect(onDone).toHaveBeenCalledWith("K7QF-M2XA-9TRE-HW4C");
});

test("RecoveryCodeNotice는 보관 확인 체크를 해야 다음 버튼이 열린다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();
  const printSpy = vi.spyOn(window, "print").mockImplementation(() => {});

  render(<RecoveryCodeNotice code="K7QF-M2XA-9TRE-HW4C" onDone={onDone} />);

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

  printSpy.mockRestore();
});
