import { afterEach, describe, expect, test, vi } from "vitest";
import { saveFile } from "./saveFile";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("saveFile", () => {
  afterEach(() => {
    invoke.mockReset();
  });

  test("앱 명령에 이름과 숫자 배열을 넘기고 저장한 경로를 돌려준다", async () => {
    invoke.mockResolvedValue("/home/직원/명단.xlsx");

    const path = await saveFile("명단.xlsx", new Uint8Array([80, 75, 3]));

    expect(path).toBe("/home/직원/명단.xlsx");
    expect(invoke).toHaveBeenCalledWith("save_file", {
      fileName: "명단.xlsx",
      contents: [80, 75, 3],
    });
  });

  test("저장 창을 닫으면 null", async () => {
    invoke.mockResolvedValue(null);

    expect(await saveFile("명단.xlsx", new Uint8Array())).toBeNull();
  });

  test("앱이 보낸 실패 문장을 그대로 오류로 던진다", async () => {
    invoke.mockRejectedValue("다른 곳을 골라 주세요");

    await expect(saveFile("명단.xlsx", new Uint8Array())).rejects.toThrow(
      "다른 곳을 골라 주세요",
    );
  });
});
