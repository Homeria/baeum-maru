import { afterEach, describe, expect, test, vi } from "vitest";
import { changeRunMode, chooseRunMode } from "./runMode";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("실행 모드 고르기", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("고른 모드를 앱 명령에 넘긴다", async () => {
    invoke.mockResolvedValue(undefined);

    await chooseRunMode("remote");

    expect(invoke).toHaveBeenCalledWith("choose_run_mode", { mode: "remote" });
  });

  test("바꿀 모드를 앱 명령에 넘긴다", async () => {
    invoke.mockResolvedValue(undefined);

    await changeRunMode("host");

    expect(invoke).toHaveBeenCalledWith("change_run_mode", { mode: "host" });
  });

  test("앱이 보낸 실패 문장을 그대로 오류로 던진다", async () => {
    invoke.mockRejectedValue("실행 모드가 이미 정해져 있습니다");

    await expect(chooseRunMode("host")).rejects.toThrow(
      "실행 모드가 이미 정해져 있습니다",
    );
  });
});
