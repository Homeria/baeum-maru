import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { remoteSettings, saveHostAddress } from "./api";
import { approvalStatus } from "./approvalApi";
import { checkConnection } from "./connection";
import { RemoteGate } from "./RemoteGate";

vi.mock("./api", () => ({
  remoteSettings: vi.fn(),
  saveHostAddress: vi.fn(),
  saveDeviceRequest: vi.fn(),
  clearDeviceRequest: vi.fn().mockResolvedValue(undefined),
  forgetHost: vi.fn().mockResolvedValue(undefined),
  rememberApproved: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("./approvalApi", () => ({
  requestApproval: vi.fn(),
  approvalStatus: vi.fn(),
}));

vi.mock("./connection", () => ({
  checkConnection: vi.fn(),
}));

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

test("기억한 호스트 주소가 없으면 주소 입력 폼을 보여 주고 연결 시 저장한다", async () => {
  const user = userEvent.setup();
  vi.mocked(remoteSettings).mockResolvedValue({
    hostAddress: null,
    deviceToken: null,
    approvalCode: null,
    computerName: "접수 노트북",
    approvedOnce: false,
  });
  vi.mocked(saveHostAddress).mockResolvedValue("192.168.0.12");

  renderWithQuery(
    <RemoteGate leftoverData={false}>
      <p>로그인 화면</p>
    </RemoteGate>,
  );

  const input = await screen.findByLabelText("호스트 주소");
  await user.type(input, "192.168.0.12");
  await user.click(screen.getByRole("button", { name: "연결" }));

  expect(saveHostAddress).toHaveBeenCalledWith(
    "192.168.0.12",
    expect.anything(),
  );
});

test("호스트에 연결하지 못하면 오류 배너를 보여 주고 주소 다시 입력으로 전환한다", async () => {
  const user = userEvent.setup();
  vi.mocked(remoteSettings).mockResolvedValue({
    hostAddress: "192.168.0.12",
    deviceToken: "tok",
    approvalCode: null,
    computerName: "접수 노트북",
    approvedOnce: true,
  });
  vi.mocked(checkConnection).mockResolvedValue({ kind: "unreachable" });

  renderWithQuery(
    <RemoteGate leftoverData={false}>
      <p>로그인 화면</p>
    </RemoteGate>,
  );

  expect(await screen.findByRole("alert")).toHaveTextContent("192.168.0.12");
  await user.click(screen.getByRole("button", { name: "주소 다시 입력" }));
  expect(screen.getByLabelText("호스트 주소")).toHaveValue("192.168.0.12");
});

test("승인 대기 상태에서는 승인 번호를 보여 주고, 준비되면 children을 렌더링한다", async () => {
  vi.mocked(remoteSettings).mockResolvedValue({
    hostAddress: "192.168.0.12",
    deviceToken: "tok",
    approvalCode: "4829",
    computerName: "접수 노트북",
    approvedOnce: false,
  });
  vi.mocked(checkConnection).mockResolvedValue({ kind: "waiting" });
  vi.mocked(approvalStatus).mockResolvedValue({ status: "pending" });

  const { unmount } = renderWithQuery(
    <RemoteGate leftoverData={false}>
      <p>로그인 화면</p>
    </RemoteGate>,
  );

  expect(await screen.findByText("4829")).toBeInTheDocument();
  expect(screen.getByRole("status")).toHaveTextContent("승인을 기다리는 중…");
  unmount();

  vi.mocked(checkConnection).mockResolvedValue({ kind: "ready" });
  renderWithQuery(
    <RemoteGate leftoverData={false}>
      <p>로그인 화면</p>
    </RemoteGate>,
  );
  expect(await screen.findByText("로그인 화면")).toBeInTheDocument();
});
