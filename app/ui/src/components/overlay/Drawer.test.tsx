import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { Button } from "../ui/Button";
import { Drawer } from "./Drawer";

test("바뀐 내용이 없으면 닫기 버튼·Esc·바깥 클릭으로 바로 닫힌다", async () => {
  const user = userEvent.setup();
  const onClose = vi.fn();

  render(
    <Drawer
      open
      title="회원 고치기"
      subtitle="회원번호 1001"
      onClose={onClose}
      footerLeft={<Button variant="danger">지우기</Button>}
      footerRight={<Button variant="primary">저장</Button>}
    >
      <p>편집 폼</p>
    </Drawer>,
  );

  const drawer = screen.getByRole("dialog", { name: "회원 고치기" });
  expect(screen.getByText("회원번호 1001")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "지우기" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "저장" })).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "닫기" }));
  expect(onClose).toHaveBeenCalledTimes(1);

  fireEvent(drawer, new Event("cancel", { cancelable: true }));
  expect(onClose).toHaveBeenCalledTimes(2);

  vi.spyOn(drawer, "getBoundingClientRect").mockReturnValue({
    left: 760,
    top: 0,
    right: 1280,
    bottom: 800,
    width: 520,
    height: 800,
    x: 760,
    y: 0,
    toJSON: () => ({}),
  });
  fireEvent.pointerDown(drawer, { clientX: 200, clientY: 200 });
  fireEvent.pointerUp(drawer, { clientX: 200, clientY: 200 });
  expect(onClose).toHaveBeenCalledTimes(3);
});

test("isDirty가 참이면 닫기 전에 확인 대화상자를 띄우고 선택에 따라 처리한다", async () => {
  const user = userEvent.setup();
  const onClose = vi.fn();

  render(
    <Drawer open isDirty title="강좌 고치기" onClose={onClose}>
      <p>강좌 폼</p>
    </Drawer>,
  );

  await user.click(screen.getByRole("button", { name: "닫기" }));
  expect(onClose).not.toHaveBeenCalled();

  expect(
    screen.getByRole("dialog", { name: "저장하지 않은 내용이 있습니다" }),
  ).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "계속 편집" })).toHaveFocus();

  await user.click(screen.getByRole("button", { name: "계속 편집" }));
  expect(
    screen.queryByRole("dialog", { name: "저장하지 않은 내용이 있습니다" }),
  ).not.toBeInTheDocument();
  expect(onClose).not.toHaveBeenCalled();

  const drawer = screen.getByRole("dialog", { name: "강좌 고치기" });
  fireEvent(drawer, new Event("cancel", { cancelable: true }));
  await user.click(screen.getByRole("button", { name: "버리고 닫기" }));
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("busy가 참이면 닫기 버튼과 Esc가 모두 막힌다", async () => {
  const user = userEvent.setup();
  const onClose = vi.fn();

  render(
    <Drawer open isDirty busy title="회원 고치기" onClose={onClose}>
      <p>저장 중…</p>
    </Drawer>,
  );

  const closeButton = screen.getByRole("button", { name: "닫기" });
  expect(closeButton).toBeDisabled();
  await user.click(closeButton);

  const drawer = screen.getByRole("dialog", { name: "회원 고치기" });
  fireEvent(drawer, new Event("cancel", { cancelable: true }));

  expect(onClose).not.toHaveBeenCalled();
  expect(
    screen.queryByRole("dialog", { name: "저장하지 않은 내용이 있습니다" }),
  ).not.toBeInTheDocument();
});
