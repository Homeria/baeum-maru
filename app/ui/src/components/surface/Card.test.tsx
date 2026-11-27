import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import { Button } from "../ui/Button";
import { Card } from "./Card";

test("제목·설명·오른쪽 동작과 본문을 함께 렌더링한다", () => {
  render(
    <Card
      title="회원 목록"
      subtitle="총 12명"
      actions={<Button variant="primary">회원 등록</Button>}
    >
      <p>목록 내용</p>
    </Card>,
  );

  expect(
    screen.getByRole("heading", { level: 2, name: "회원 목록" }),
  ).toBeInTheDocument();
  expect(screen.getByText("총 12명")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "회원 등록" })).toBeInTheDocument();
  expect(screen.getByText("목록 내용")).toBeInTheDocument();
});

test("headingLevel=3을 주면 h3 제목을 쓴다", () => {
  render(
    <Card title="우선 배정 사유" headingLevel={3}>
      <p>내용</p>
    </Card>,
  );

  expect(
    screen.getByRole("heading", { level: 3, name: "우선 배정 사유" }),
  ).toBeInTheDocument();
});
