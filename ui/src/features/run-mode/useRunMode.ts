import { useQuery } from "@tanstack/react-query";
import { runModeState } from "../../api/runMode";

/** 켤 때 정한 실행 모드(ADR 0021). 켤 때 한 번 정해지고 바뀌지 않는다. */
export function useRunMode() {
  return useQuery({
    queryKey: ["runModeState"],
    queryFn: runModeState,
    staleTime: Infinity,
  });
}

/**
 * 이 창이 호스트 PC의 창인지. 승인 요청 허락처럼 호스트 PC에서만 하는 일에 쓴다.
 *
 * 앱에 묻지 못했으면(예: 앱 밖 브라우저) 호스트로 본다. 실제로 막는 것은 호스트다.
 */
export function useIsHostPc(): boolean {
  const runMode = useRunMode();
  return runMode.data?.mode === "host" || runMode.isError;
}
