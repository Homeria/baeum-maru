import { afterEach, describe, expect, test, vi } from "vitest";
import { previewBrokenDbBackup, restoreBrokenDb } from "./api";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("깨진 DB 복원 명령", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("목록 이름이나 경로 중 하나만 채워 보낸다", async () => {
    invoke.mockResolvedValue(undefined);

    await restoreBrokenDb({ fileName: "20261122-090000_daily.db" });
    await previewBrokenDbBackup({ path: "E:\\백업\\옛 백업.db" });

    expect(invoke).toHaveBeenNthCalledWith(1, "restore_broken_db", {
      request: { fileName: "20261122-090000_daily.db", path: null },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "preview_broken_db_backup", {
      request: { fileName: null, path: "E:\\백업\\옛 백업.db" },
    });
  });

  test("앱이 보낸 실패 문장을 그대로 오류로 던진다", async () => {
    invoke.mockRejectedValue("배움마루 백업 파일이 아닙니다");

    await expect(restoreBrokenDb({ path: "C:\\a.db" })).rejects.toThrow(
      "배움마루 백업 파일이 아닙니다",
    );
  });
});
