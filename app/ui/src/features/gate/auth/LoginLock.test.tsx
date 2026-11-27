import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { HostError } from "../../../api/client";
import type { Session } from "../../../api/session";
import { listLoginStaff, login, logout, unlock } from "./api";
import { LockScreen } from "./LockScreen";
import { LoginPage } from "./LoginPage";

vi.mock("./api", () => ({
  listLoginStaff: vi.fn(),
  login: vi.fn(),
  logout: vi.fn(),
  unlock: vi.fn(),
}));

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

test("LoginPage는 역할별 직원 목록을 보여 주고 이름 선택 뒤 PIN으로 로그인한다", async () => {
  const user = userEvent.setup();
  const onForgotPin = vi.fn();
  vi.mocked(listLoginStaff).mockResolvedValue({
    items: [
      { id: 1, name: "관리자1", role: "admin" },
      { id: 2, name: "직원1", role: "staff" },
    ],
  });
  vi.mocked(login).mockResolvedValue({
    token: "tok",
    staff: { id: 2, name: "직원1", role: "staff" },
  });

  renderWithQuery(<LoginPage onForgotPin={onForgotPin} />);

  await user.click(await screen.findByText("직원1"));
  const pinInput = screen.getByLabelText("PIN");
  expect(pinInput).toHaveFocus();

  await user.type(pinInput, "1234");
  await user.click(screen.getByRole("button", { name: "로그인" }));
  expect(login).toHaveBeenCalledWith(
    { staffId: 2, pin: "1234" },
    expect.anything(),
  );

  await user.click(screen.getByRole("button", { name: "PIN을 잊었어요" }));
  expect(onForgotPin).toHaveBeenCalledTimes(1);
});

test("LoginPage는 PIN이 틀리면 입력 칸을 비우고 오류 알림을 보여 준다", async () => {
  const user = userEvent.setup();
  vi.mocked(listLoginStaff).mockResolvedValue({
    items: [{ id: 1, name: "관리자1", role: "admin" }],
  });
  vi.mocked(login).mockRejectedValue(
    new HostError(401, "PIN이 맞지 않습니다", "WRONG_PIN"),
  );

  renderWithQuery(<LoginPage onForgotPin={() => {}} />);

  await user.click(await screen.findByText("관리자1"));
  const pinInput = screen.getByLabelText("PIN");
  await user.type(pinInput, "000000");
  await user.click(screen.getByRole("button", { name: "로그인" }));

  expect(await screen.findByRole("alert")).toHaveTextContent(
    "PIN이 맞지 않습니다",
  );
  expect(pinInput).toHaveValue("");
  expect(pinInput).toHaveFocus();
});

test("LockScreen은 Esc로 닫히지 않으며 PIN 해제와 다른 직원 로그인을 처리한다", async () => {
  const user = userEvent.setup();
  vi.mocked(unlock).mockResolvedValue(undefined);
  vi.mocked(logout).mockResolvedValue(undefined);
  const session: Session = {
    token: "tok",
    staff: { id: 1, name: "관리자1", role: "admin" },
    locked: true,
  };

  renderWithQuery(<LockScreen session={session} />);

  const dialog = screen.getByRole("dialog", { name: "화면이 잠겼습니다" });
  const pinInput = screen.getByLabelText("PIN");
  expect(pinInput).toHaveFocus();

  const cancelEvent = new Event("cancel", { cancelable: true });
  fireEvent(dialog, cancelEvent);
  expect(cancelEvent.defaultPrevented).toBe(true);

  await user.type(pinInput, "123456");
  await user.click(screen.getByRole("button", { name: "열기" }));
  expect(unlock).toHaveBeenCalledWith("123456", expect.anything());

  await user.click(
    screen.getByRole("button", { name: "다른 직원으로 로그인" }),
  );
  expect(logout).toHaveBeenCalledTimes(1);
});
