import { StatusPill } from "../../../components/ui/StatusPill";

/** 역할상 바꿀 수 없을 때 화면 머리줄에 보여 주는 꼬리표 (BR-K3). */
export function ViewOnlyNotice() {
  return (
    <span role="status">
      <StatusPill tone="neutral">
        지금 역할로는 보기만 할 수 있습니다
      </StatusPill>
    </span>
  );
}
