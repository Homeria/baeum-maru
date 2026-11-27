import { afterEach, describe, expect, test, vi } from "vitest";
import { hostAddress, openDeviceRequestCount } from "./api";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("호스트 화면의 앱 명령", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("접속 주소와 승인 요청 수는 서버가 아니라 앱에 묻는다", async () => {
    invoke.mockResolvedValueOnce("192.168.0.12").mockResolvedValueOnce(2);

    expect(await hostAddress()).toBe("192.168.0.12");
    expect(await openDeviceRequestCount()).toBe(2);
    expect(invoke).toHaveBeenNthCalledWith(1, "host_address");
    expect(invoke).toHaveBeenNthCalledWith(2, "open_device_request_count");
  });
});
