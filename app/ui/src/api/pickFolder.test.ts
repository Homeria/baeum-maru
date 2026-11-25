import { afterEach, describe, expect, test, vi } from "vitest";
import { pickFolder } from "./pickFolder";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("pickFolder", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("앱 명령이 돌려준 폴더 경로를 돌려준다", async () => {
    invoke.mockResolvedValue("E:\\배움마루 백업");

    expect(await pickFolder()).toBe("E:\\배움마루 백업");
    expect(invoke).toHaveBeenCalledWith("pick_folder");
  });

  test("창을 닫으면 null", async () => {
    invoke.mockResolvedValue(null);

    expect(await pickFolder()).toBeNull();
  });

  test("앱이 보낸 실패 문장을 그대로 오류로 던진다", async () => {
    invoke.mockRejectedValue("고른 폴더를 알 수 없습니다");

    await expect(pickFolder()).rejects.toThrow("고른 폴더를 알 수 없습니다");
  });
});
