import { useQuery } from "@tanstack/react-query";
import { openDeviceRequestCount } from "../../api/devices";

/** 승인 요청 수를 다시 묻는 간격. */
const CHECK_EVERY_MS = 5 * 1000;

/**
 * 기다리는 승인 요청 수(BR-K15). `active`일 때만 주기적으로 앱에 묻는다.
 *
 * 앱에 물어서 로그인 세션을 건드리지 않는다. 변경 알림(SSE)이 생기면 그것으로 바꾼다.
 */
export function useOpenRequestCount(active: boolean): number {
  const { data } = useQuery({
    queryKey: ["openDeviceRequestCount"],
    queryFn: openDeviceRequestCount,
    enabled: active,
    refetchInterval: CHECK_EVERY_MS,
  });
  return active ? (data ?? 0) : 0;
}
