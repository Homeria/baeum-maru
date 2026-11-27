import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import { StatusPill, type StatusPillTone } from "./StatusPill";

test("상태 글자를 꼬리표로 보여 준다", () => {
  render(<StatusPill tone="ok">사용 중</StatusPill>);

  expect(screen.getByText("사용 중")).toBeInTheDocument();
});

test("다섯 가지 상태 성격이 모두 렌더링된다", () => {
  const items: readonly { tone: StatusPillTone; text: string }[] = [
    { tone: "neutral", text: "신청함" },
    { tone: "primary", text: "우선" },
    { tone: "ok", text: "연결됨" },
    { tone: "bad", text: "필수" },
    { tone: "turn", text: "연락할 차례" },
  ];

  render(
    <div>
      {items.map((item) => (
        <StatusPill key={item.tone} tone={item.tone}>
          {item.text}
        </StatusPill>
      ))}
    </div>,
  );

  for (const item of items) {
    expect(screen.getByText(item.text)).toBeInTheDocument();
  }
});
