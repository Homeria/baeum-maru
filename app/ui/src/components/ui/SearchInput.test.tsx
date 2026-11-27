import { useState } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test } from "vitest";
import { SearchInput } from "./SearchInput";

function ControlledSearch({ hideLabel = false }: { hideLabel?: boolean }) {
  const [value, setValue] = useState("");
  return (
    <SearchInput
      label="회원 검색"
      hideLabel={hideLabel}
      placeholder="이름, 연락처 뒷자리"
      value={value}
      onChange={setValue}
    />
  );
}

test("검색어를 입력하면 지우기 버튼이 나타나고 누르면 비워지며 초점이 돌아간다", async () => {
  const user = userEvent.setup();
  render(<ControlledSearch />);

  const input = screen.getByRole("searchbox", { name: "회원 검색" });
  expect(
    screen.queryByRole("button", { name: "검색어 지우기" }),
  ).not.toBeInTheDocument();

  await user.type(input, "김영희");
  expect(input).toHaveValue("김영희");

  const clearButton = screen.getByRole("button", { name: "검색어 지우기" });
  await user.click(clearButton);

  expect(input).toHaveValue("");
  expect(input).toHaveFocus();
  expect(
    screen.queryByRole("button", { name: "검색어 지우기" }),
  ).not.toBeInTheDocument();
});

test("hideLabel이 참이어도 접근성 이름으로 검색 칸을 찾을 수 있다", () => {
  render(<ControlledSearch hideLabel />);

  expect(
    screen.getByRole("searchbox", { name: "회원 검색" }),
  ).toBeInTheDocument();
});
