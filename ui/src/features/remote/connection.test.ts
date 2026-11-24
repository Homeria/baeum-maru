import { afterEach, describe, expect, test, vi } from "vitest";
import { HostError } from "../../api/client";
import { checkConnection } from "./connection";

const mocks = vi.hoisted(() => ({
  fetchHostVersion: vi.fn(),
  approvalStatus: vi.fn(),
  clearDeviceRequest: vi.fn(),
  connectToHost: vi.fn(),
}));
vi.mock("../../api/version", () => ({
  fetchHostVersion: mocks.fetchHostVersion,
}));
vi.mock("../../api/approval", () => ({
  approvalStatus: mocks.approvalStatus,
}));
vi.mock("../../api/remote", () => ({
  clearDeviceRequest: mocks.clearDeviceRequest,
}));
vi.mock("../../api/client", async (original) => ({
  ...(await original<typeof import("../../api/client")>()),
  connectToHost: mocks.connectToHost,
}));

const compatible = { hostVersion: "0.1.0", compatible: true };

describe("checkConnection", () => {
  afterEach(() => {
    vi.resetAllMocks();
  });

  test("요청을 기억한 주소와 증표로 보내게 맞춘다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);
    const onRejected = () => {};

    await checkConnection("192.168.0.12", null, onRejected);

    expect(mocks.connectToHost).toHaveBeenCalledWith(
      "192.168.0.12",
      null,
      onRejected,
    );
  });

  test("호스트에 닿지 않으면 승인을 묻지 않는다", async () => {
    mocks.fetchHostVersion.mockRejectedValue(new HostError(0, "연결 안 됨"));

    expect(await checkConnection("192.168.0.12", "증표", () => {})).toEqual({
      kind: "unreachable",
    });
    expect(mocks.approvalStatus).not.toHaveBeenCalled();
  });

  test("버전이 맞지 않으면 알린다", async () => {
    mocks.fetchHostVersion.mockResolvedValue({
      hostVersion: "0.2.0",
      compatible: false,
    });

    expect(await checkConnection("192.168.0.12", "증표", () => {})).toEqual({
      kind: "incompatible",
      hostVersion: "0.2.0",
    });
  });

  test("증표가 없으면 승인을 요청한다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);

    expect(await checkConnection("192.168.0.12", null, () => {})).toEqual({
      kind: "request",
    });
  });

  test("승인 상태를 따른다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);
    mocks.approvalStatus
      .mockResolvedValueOnce({ status: "approved" })
      .mockResolvedValueOnce({ status: "pending" });

    expect(await checkConnection("h", "증표", () => {})).toEqual({
      kind: "ready",
    });
    expect(await checkConnection("h", "증표", () => {})).toEqual({
      kind: "waiting",
    });
    expect(mocks.clearDeviceRequest).not.toHaveBeenCalled();
  });

  test("거절되었거나 5분이 지났으면 증표를 버린다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);
    mocks.approvalStatus.mockResolvedValue({ status: "expired" });

    expect(await checkConnection("h", "증표", () => {})).toEqual({
      kind: "refused",
      reason: "expired",
    });
    expect(mocks.clearDeviceRequest).toHaveBeenCalledOnce();
  });

  test("호스트가 모르는 증표면 버리고 다시 요청한다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);
    mocks.approvalStatus.mockRejectedValue(
      new HostError(409, "없음", "APPROVAL_REQUEST_NOT_OPEN"),
    );

    expect(await checkConnection("h", "증표", () => {})).toEqual({
      kind: "request",
    });
    expect(mocks.clearDeviceRequest).toHaveBeenCalledOnce();
  });
});
