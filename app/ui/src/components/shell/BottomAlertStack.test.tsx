import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { BottomAlertStack, type BottomAlertItem } from "./BottomAlertStack";

test("알림이 없으면 아무것도 렌더링하지 않는다", () => {
  const { container } = render(<BottomAlertStack items={[]} />);

  expect(container).toBeEmptyDOMElement();
});

test("위험 알림은 alert, 정보·일반 알림은 status 역할로 표시하고 동작·닫기 버튼을 연결한다", async () => {
  const user = userEvent.setup();
  const onCheck = vi.fn();
  const onDismiss = vi.fn();

  const items: readonly BottomAlertItem[] = [
    {
      id: "conn",
      tone: "danger",
      message: "호스트 연결이 끊겼습니다. 다시 연결하는 중입니다.",
    },
    {
      id: "device",
      tone: "info",
      message: "새 기기 승인 요청 1건이 있습니다.",
      actionLabel: "확인하기",
      onAction: onCheck,
      onDismiss,
    },
    {
      id: "view-only",
      tone: "neutral",
      message: "보기 전용 PC입니다.",
    },
  ];

  render(<BottomAlertStack items={items} />);

  expect(screen.getByRole("alert")).toHaveTextContent(
    "호스트 연결이 끊겼습니다.",
  );
  const statuses = screen.getAllByRole("status");
  expect(statuses).toHaveLength(2);
  expect(statuses[0]).toHaveTextContent("새 기기 승인 요청 1건이 있습니다.");
  expect(statuses[1]).toHaveTextContent("보기 전용 PC입니다.");

  await user.click(screen.getByRole("button", { name: "확인하기" }));
  expect(onCheck).toHaveBeenCalledTimes(1);

  await user.click(screen.getByRole("button", { name: "닫기" }));
  expect(onDismiss).toHaveBeenCalledTimes(1);
});
