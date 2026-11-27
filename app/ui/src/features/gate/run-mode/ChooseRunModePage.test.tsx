import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { chooseRunMode } from "../../../api/runMode";
import { listLoginStaff } from "../auth/api";
import { ChangeRunModeDialog } from "./ChangeRunModeDialog";
import { ChooseRunModePage } from "./ChooseRunModePage";

vi.mock("../../../api/runMode", () => ({
  chooseRunMode: vi.fn(),
  changeRunMode: vi.fn(),
  permitRunModeChange: vi.fn(),
}));

vi.mock("../auth/api", () => ({
  listLoginStaff: vi.fn(),
}));

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

test("모드를 고르지 않고 시작을 누르면 오류를 보여 주고, 고르면 chooseRunMode를 부른다", async () => {
  const user = userEvent.setup();
  vi.mocked(chooseRunMode).mockResolvedValue(undefined);

  renderWithQuery(<ChooseRunModePage />);

  await user.click(screen.getByRole("button", { name: "이렇게 시작" }));
  expect(screen.getByRole("alert")).toHaveTextContent("하나를 골라 주세요");
  expect(chooseRunMode).not.toHaveBeenCalled();

  await user.click(screen.getByText("이 PC에서 새로 시작"));
  await user.click(screen.getByRole("button", { name: "이렇게 시작" }));
  expect(chooseRunMode).toHaveBeenCalledWith("host", expect.anything());
});

test("넘겨받은 파일로 호스트 시작을 고르면 넘겨받기 화면으로 바뀐다", async () => {
  const user = userEvent.setup();

  renderWithQuery(<ChooseRunModePage />);

  await user.click(screen.getByText("넘겨받은 파일로 호스트 시작"));
  await user.click(screen.getByRole("button", { name: "이렇게 시작" }));

  expect(
    screen.getByRole("button", { name: "넘기기 파일 고르기" }),
  ).toBeInTheDocument();
});

test("ChangeRunModeDialog는 PIN이 필요할 때 비운 채 누르면 오류를 보여 준다", async () => {
  const user = userEvent.setup();
  vi.mocked(listLoginStaff).mockResolvedValue({
    items: [{ id: 1, name: "관리자1", role: "admin" }],
  });

  renderWithQuery(
    <ChangeRunModeDialog target="remote" needsPin onCancel={() => {}} />,
  );

  expect(await screen.findByText("관리자1")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "그만두기" })).toHaveFocus();

  await user.click(screen.getByRole("button", { name: "바꾸고 다시 켜기" }));
  expect(screen.getByLabelText("PIN")).toHaveAttribute("aria-invalid", "true");
  expect(screen.getByRole("alert")).toHaveTextContent(
    "관리자를 고르고 PIN을 넣어 주세요",
  );
});
