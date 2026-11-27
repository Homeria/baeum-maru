import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { Button } from "./Button";

test("기본 type은 button이며 클릭하면 onClick을 부른다", async () => {
  const user = userEvent.setup();
  const onClick = vi.fn();

  render(<Button onClick={onClick}>저장</Button>);

  const button = screen.getByRole("button", { name: "저장" });
  expect(button).toHaveAttribute("type", "button");

  await user.click(button);
  expect(onClick).toHaveBeenCalledTimes(1);
});

test("type=submit으로 지정하면 폼을 제출할 수 있다", () => {
  render(
    <Button type="submit" variant="primary">
      신청하기
    </Button>,
  );

  expect(screen.getByRole("button", { name: "신청하기" })).toHaveAttribute(
    "type",
    "submit",
  );
});

test("busy가 참이면 비활성화되고 busyLabel과 aria-busy를 표시한다", async () => {
  const user = userEvent.setup();
  const onClick = vi.fn();

  render(
    <Button variant="dangerSolid" busy busyLabel="지우는 중…" onClick={onClick}>
      지우기
    </Button>,
  );

  const button = screen.getByRole("button", { name: "지우는 중…" });
  expect(button).toBeDisabled();
  expect(button).toHaveAttribute("aria-busy", "true");

  await user.click(button);
  expect(onClick).not.toHaveBeenCalled();
});

test("disabled 상태에서는 클릭이 동작하지 않는다", async () => {
  const user = userEvent.setup();
  const onClick = vi.fn();

  render(
    <Button size="sm" disabled onClick={onClick}>
      고치기
    </Button>,
  );

  const button = screen.getByRole("button", { name: "고치기" });
  expect(button).toBeDisabled();
  await user.click(button);
  expect(onClick).not.toHaveBeenCalled();
});
