import { afterEach, describe, expect, test, vi } from "vitest";
import { pickBackupFile, restartApp } from "./appCommands";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("pickBackupFile", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("앱 명령이 돌려준 파일 경로를 돌려준다", async () => {
    invoke.mockResolvedValue("E:\\배움마루 백업\\20261120-090000_daily.db");

    expect(await pickBackupFile()).toBe(
      "E:\\배움마루 백업\\20261120-090000_daily.db",
    );
    expect(invoke).toHaveBeenCalledWith("pick_backup_file");
  });

  test("창을 닫으면 null", async () => {
    invoke.mockResolvedValue(null);

    expect(await pickBackupFile()).toBeNull();
  });
});

describe("restartApp", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("다시 켜지 못하면 직접 껐다 켜라고 알린다", async () => {
    invoke.mockRejectedValue("failed");

    await expect(restartApp()).rejects.toThrow("직접 껐다 켜면 복원됩니다");
    expect(invoke).toHaveBeenCalledWith("restart_app");
  });
});
