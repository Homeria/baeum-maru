import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { Tabs } from "./Tabs.tsx";

const ITEMS = [
  { key: "subjects", label: "과목" },
  { key: "rooms", label: "강의실" },
  { key: "reasons", label: "우선 배정 사유" },
] as const;

test("탭 목록과 선택한 탭의 aria-selected를 표시한다", () => {
  render(
    <Tabs
      label="기초 정보"
      items={ITEMS}
      selected="subjects"
      onSelect={() => {}}
    />,
  );

  expect(
    screen.getByRole("tablist", { name: "기초 정보" }),
  ).toBeInTheDocument();
  expect(screen.getByRole("tab", { name: "과목" })).toHaveAttribute(
    "aria-selected",
    "true",
  );
  expect(screen.getByRole("tab", { name: "강의실" })).toHaveAttribute(
    "aria-selected",
    "false",
  );
  expect(screen.getByRole("tab", { name: "우선 배정 사유" })).toHaveAttribute(
    "aria-selected",
    "false",
  );
});

test("다른 탭을 클릭하면 그 키로 onSelect를 부른다", async () => {
  const user = userEvent.setup();
  const onSelect = vi.fn();

  render(<Tabs items={ITEMS} selected="subjects" onSelect={onSelect} />);

  await user.click(screen.getByRole("tab", { name: "강의실" }));
  expect(onSelect).toHaveBeenCalledWith("rooms");
});

test("키보드 Enter로 탭을 고를 수 있다", async () => {
  const user = userEvent.setup();
  const onSelect = vi.fn();

  render(<Tabs items={ITEMS} selected="subjects" onSelect={onSelect} />);

  screen.getByRole("tab", { name: "우선 배정 사유" }).focus();
  await user.keyboard("{Enter}");
  expect(onSelect).toHaveBeenCalledWith("reasons");
});
