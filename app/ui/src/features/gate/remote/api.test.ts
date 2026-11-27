import { afterEach, describe, expect, test, vi } from "vitest";
import { clearDeviceRequest, saveDeviceRequest, saveHostAddress } from "./api";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("접속 설정 앱 명령", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("주소와 기기 증표를 앱에 맡긴다", async () => {
    invoke.mockResolvedValueOnce("192.168.0.12").mockResolvedValue(undefined);

    expect(await saveHostAddress("http://192.168.0.12")).toBe("192.168.0.12");
    await saveDeviceRequest("증표", "123456");
    await clearDeviceRequest();

    expect(invoke).toHaveBeenNthCalledWith(1, "save_host_address", {
      address: "http://192.168.0.12",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "save_device_request", {
      token: "증표",
      code: "123456",
    });
    expect(invoke).toHaveBeenNthCalledWith(
      3,
      "clear_device_request",
      undefined,
    );
  });

  test("앱이 보낸 실패 문장을 그대로 오류로 던진다", async () => {
    invoke.mockRejectedValue("호스트 주소를 넣어 주세요");

    await expect(saveHostAddress(" ")).rejects.toThrow(
      "호스트 주소를 넣어 주세요",
    );
  });
});
