import { approvalStatus } from "../../api/approval";
import { HostError, connectToHost } from "../../api/client";
import {
  checkHostIdentity,
  clearDeviceRequest,
  rememberHostIdentity,
} from "../../api/remote";
import { fetchHostVersion } from "../../api/version";

/** 접속 모드에서 호스트와 이어진 정도. */
export type Connection =
  /** 호스트에 닿지 않는다. 꺼져 있거나 주소가 바뀌었다. */
  | { kind: "unreachable" }
  /** 호스트와 앱의 버전이 맞지 않는다. */
  | { kind: "incompatible"; hostVersion: string }
  /** 이 PC가 쓰던 데이터의 넘기기 전 옛 호스트다(BR-H5). 붙지 않는다. */
  | { kind: "olderHost" }
  /** 이 PC가 쓰던 것과 다른 데이터의 호스트다(BR-H5). 붙지 않는다. */
  | { kind: "otherHost" }
  /** 이 PC는 아직 승인을 요청하지 않았다. */
  | { kind: "request" }
  /** 승인을 기다린다. */
  | { kind: "waiting" }
  /** 거절되었거나 5분이 지났다. 기기 증표는 버렸다. */
  | { kind: "refused"; reason: "rejected" | "expired" }
  /** 승인받았다. 로그인할 수 있다. */
  | { kind: "ready" };

/**
 * 기억한 주소의 호스트에 이 PC가 어디까지 이어졌는지 본다(ADR 0004, BR-K14, BR-K15).
 *
 * 앞으로의 요청이 그 호스트로 가고 기기 증표를 싣도록 맞춘 뒤, 호스트가 응답하는지,
 * 버전이 맞는지, 쓰던 데이터의 최신 호스트인지(BR-H5), 승인받았는지를 차례로 묻는다.
 * 승인받았으면 그 호스트를 기억한다. `onRejected`는 쓰는 중에 승인이 취소되면 불린다.
 */
export async function checkConnection(
  address: string,
  token: string | null,
  onRejected: () => void,
): Promise<Connection> {
  connectToHost(address, token, onRejected);
  let version;
  try {
    version = await fetchHostVersion();
  } catch {
    return { kind: "unreachable" };
  }
  if (!version.compatible) {
    return { kind: "incompatible", hostVersion: version.hostVersion };
  }
  const host = await checkHostIdentity(version.bundleId, version.generation);
  if (host === "older") {
    return { kind: "olderHost" };
  }
  if (host === "other_bundle") {
    return { kind: "otherHost" };
  }
  if (token === null) {
    return { kind: "request" };
  }
  let status;
  try {
    ({ status } = await approvalStatus());
  } catch (error) {
    if (
      error instanceof HostError &&
      error.code === "APPROVAL_REQUEST_NOT_OPEN"
    ) {
      // 이 호스트가 모르는 증표다. 다른 호스트로 주소를 바꿨을 수 있다
      await clearDeviceRequest();
      return { kind: "request" };
    }
    throw error;
  }
  switch (status) {
    case "approved":
      // 넘겨받은 새 호스트면 새 세대를 기억해, 옛 호스트에 다시 붙지 않게 한다
      await rememberHostIdentity(version.bundleId, version.generation);
      return { kind: "ready" };
    case "pending":
      return { kind: "waiting" };
    case "rejected":
    case "expired":
      await clearDeviceRequest();
      return { kind: "refused", reason: status };
  }
}
