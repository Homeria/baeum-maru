import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { HostError } from "../../../api/client";
import { checkConnection } from "./connection";

const mocks = vi.hoisted(() => ({
  fetchHostVersion: vi.fn(),
  approvalStatus: vi.fn(),
  clearDeviceRequest: vi.fn(),
  checkHostIdentity: vi.fn(),
  rememberHostIdentity: vi.fn(),
  connectToHost: vi.fn(),
}));
vi.mock("../../../api/version", () => ({
  fetchHostVersion: mocks.fetchHostVersion,
}));
vi.mock("./approvalApi", () => ({
  approvalStatus: mocks.approvalStatus,
}));
vi.mock("./api", () => ({
  clearDeviceRequest: mocks.clearDeviceRequest,
  checkHostIdentity: mocks.checkHostIdentity,
  rememberHostIdentity: mocks.rememberHostIdentity,
}));
vi.mock("../../../api/client", async (original) => ({
  ...(await original<typeof import("../../../api/client")>()),
  connectToHost: mocks.connectToHost,
}));

const bundleId = "00112233445566778899aabbccddeeff";
const compatible = {
  hostVersion: "0.1.0",
  compatible: true,
  bundleId,
  generation: 2,
};

describe("checkConnection", () => {
  beforeEach(() => {
    mocks.checkHostIdentity.mockResolvedValue("same");
  });

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
      ...compatible,
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

  test("승인받았으면 그 호스트를 기억한다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);
    mocks.approvalStatus.mockResolvedValue({ status: "approved" });

    await checkConnection("h", "증표", () => {});

    expect(mocks.checkHostIdentity).toHaveBeenCalledWith(bundleId, 2);
    expect(mocks.rememberHostIdentity).toHaveBeenCalledWith(bundleId, 2);
  });

  test("승인받기 전에는 호스트를 기억하지 않는다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);
    mocks.approvalStatus.mockResolvedValue({ status: "pending" });

    await checkConnection("h", "증표", () => {});
    await checkConnection("h", null, () => {});

    expect(mocks.rememberHostIdentity).not.toHaveBeenCalled();
  });

  test("BR-H5 옛 호스트와 다른 데이터의 호스트에는 붙지 않는다", async () => {
    mocks.fetchHostVersion.mockResolvedValue(compatible);
    mocks.checkHostIdentity
      .mockResolvedValueOnce("older")
      .mockResolvedValueOnce("other_bundle");

    expect(await checkConnection("h", "증표", () => {})).toEqual({
      kind: "olderHost",
    });
    expect(await checkConnection("h", "증표", () => {})).toEqual({
      kind: "otherHost",
    });
    expect(mocks.approvalStatus).not.toHaveBeenCalled();
    expect(mocks.clearDeviceRequest).not.toHaveBeenCalled();
  });
});
