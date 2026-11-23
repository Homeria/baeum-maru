import { invoke } from "@tauri-apps/api/core";

/**
 * 이 PC에 파일을 저장한다(ADR 0018). 앱이 저장 창을 띄우고 직원이 폴더와 이름을 고른다.
 *
 * 저장한 파일 경로를 돌려준다. 직원이 저장 창을 닫으면 `null`.
 * 저장하지 못하면 직원에게 보여 줄 문장을 담은 `Error`를 던진다.
 *
 * @param fileName 저장 창에 미리 채울 이름. 예: `2027년 1학기 원본 명단.xlsx`
 */
export async function saveFile(
  fileName: string,
  contents: Uint8Array,
): Promise<string | null> {
  try {
    return await invoke<string | null>("save_file", {
      fileName,
      // 앱 명령은 JSON으로 받는다. Uint8Array는 숫자 배열로 바꿔 보낸다
      contents: Array.from(contents),
    });
  } catch (failure) {
    throw new Error(
      typeof failure === "string" ? failure : "파일을 저장하지 못했습니다",
      { cause: failure },
    );
  }
}
