import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { Button } from "../ui/Button";
import { StatusPill } from "../ui/StatusPill";
import { AppShell, type AppShellNavSection } from "./AppShell";

type SampleMenu =
  "members" | "courses" | "applications" | "referenceData" | "staff";

const SECTIONS: readonly AppShellNavSection<SampleMenu>[] = [
  {
    label: "수강 사무",
    items: [
      { key: "members", label: "회원" },
      { key: "courses", label: "강좌" },
      { key: "applications", label: "신청" },
    ],
  },
  {
    label: "기관 운영",
    items: [
      { key: "referenceData", label: "기초 정보" },
      {
        key: "staff",
        label: "직원 관리",
        badge: <StatusPill tone="primary">1</StatusPill>,
      },
    ],
  },
];

test("왼쪽 메뉴 묶음·현재 메뉴(aria-current=page)·머리줄·하단 알림을 렌더링한다", async () => {
  const user = userEvent.setup();
  const onSelect = vi.fn();

  render(
    <AppShell
      sections={SECTIONS}
      selected="members"
      onSelect={onSelect}
      pageTitle="회원"
      hostStatus={<StatusPill tone="ok">호스트 연결됨</StatusPill>}
      headerStatus={
        <StatusPill tone="neutral">
          지금 역할로는 보기만 할 수 있습니다
        </StatusPill>
      }
      headerActions={<Button variant="primary">회원 등록</Button>}
      sidebarFooter={<p>관리자 홍길동</p>}
      alerts={[
        {
          id: "conn",
          tone: "danger",
          message: "호스트 연결이 끊겼습니다.",
        },
      ]}
    >
      <p>회원 목록 본문</p>
    </AppShell>,
  );

  const nav = screen.getByRole("navigation", { name: "업무 메뉴" });
  expect(within(nav).getByText("수강 사무")).toBeInTheDocument();
  expect(within(nav).getByText("기관 운영")).toBeInTheDocument();
  expect(within(nav).getByRole("button", { name: "회원" })).toHaveAttribute(
    "aria-current",
    "page",
  );

  await user.click(within(nav).getByRole("button", { name: "강좌" }));
  expect(onSelect).toHaveBeenCalledWith("courses");

  expect(
    screen.getByRole("heading", { level: 1, name: "회원" }),
  ).toBeInTheDocument();
  expect(screen.getByText("호스트 연결됨")).toBeInTheDocument();
  expect(
    screen.getByText("지금 역할로는 보기만 할 수 있습니다"),
  ).toBeInTheDocument();
  expect(screen.getByRole("alert")).toHaveTextContent(
    "호스트 연결이 끊겼습니다.",
  );
});

test("메뉴 접기를 누르면 왼쪽 메뉴가 숨겨지고 메뉴 열기 버튼으로 초점이 이동한다", async () => {
  const user = userEvent.setup();

  render(
    <AppShell
      sections={SECTIONS}
      selected="members"
      onSelect={() => {}}
      pageTitle="회원"
    >
      <p>본문</p>
    </AppShell>,
  );

  await user.click(screen.getByRole("button", { name: "메뉴 접기" }));

  expect(
    screen.queryByRole("navigation", { name: "업무 메뉴" }),
  ).not.toBeInTheDocument();

  const expandButton = screen.getByRole("button", { name: "메뉴 열기" });
  expect(expandButton).toHaveAttribute("aria-expanded", "false");
  await waitFor(() => expect(expandButton).toHaveFocus());

  await user.click(expandButton);

  expect(
    screen.getByRole("navigation", { name: "업무 메뉴" }),
  ).toBeInTheDocument();
  const collapseButton = screen.getByRole("button", { name: "메뉴 접기" });
  expect(collapseButton).toHaveAttribute("aria-expanded", "true");
  await waitFor(() => expect(collapseButton).toHaveFocus());
});
