import { useRef } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { Button } from "../ui/Button";
import { ModalDialog } from "./ModalDialog";

function DeleteConfirmHarness() {
  const cancelRef = useRef<HTMLButtonElement>(null);
  return (
    <ModalDialog
      open
      size="sm"
      title="회원을 지울까요?"
      initialFocusRef={cancelRef}
      onClose={() => {}}
      footer={
        <>
          <Button ref={cancelRef}>그만두기</Button>
          <Button variant="dangerSolid">지우기</Button>
        </>
      }
    >
      <p>지우면 되돌릴 수 없습니다.</p>
    </ModalDialog>
  );
}

test("open이 참이면 모달 대화상자를 열고 안전한 기본 버튼에 초점을 둔다", () => {
  render(<DeleteConfirmHarness />);

  const dialog = screen.getByRole("dialog", { name: "회원을 지울까요?" });
  expect(dialog).toBeInTheDocument();
  expect(dialog).toHaveAttribute("open");
  expect(screen.getByRole("button", { name: "그만두기" })).toHaveFocus();
});

test("Esc(cancel)를 누르면 onClose를 부르고 busy가 참이면 닫히지 않는다", () => {
  const onClose = vi.fn();

  const { rerender } = render(
    <ModalDialog open title="강좌 마감" onClose={onClose}>
      <p>마감하면 다시 추첨할 수 없습니다.</p>
    </ModalDialog>,
  );

  const dialog = screen.getByRole("dialog", { name: "강좌 마감" });
  fireEvent(dialog, new Event("cancel", { cancelable: true }));
  expect(onClose).toHaveBeenCalledTimes(1);

  rerender(
    <ModalDialog open busy title="강좌 마감" onClose={onClose}>
      <p>마감하는 중…</p>
    </ModalDialog>,
  );

  fireEvent(dialog, new Event("cancel", { cancelable: true }));
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("lightDismiss가 참이면 바깥 영역 클릭으로 닫히고 안쪽 클릭은 닫히지 않는다", async () => {
  const user = userEvent.setup();
  const onClose = vi.fn();

  render(
    <ModalDialog open lightDismiss title="안내" onClose={onClose}>
      <p>본문</p>
    </ModalDialog>,
  );

  const dialog = screen.getByRole("dialog", { name: "안내" });
  vi.spyOn(dialog, "getBoundingClientRect").mockReturnValue({
    left: 100,
    top: 100,
    right: 500,
    bottom: 400,
    width: 400,
    height: 300,
    x: 100,
    y: 100,
    toJSON: () => ({}),
  });

  await user.click(screen.getByText("본문"));
  expect(onClose).not.toHaveBeenCalled();

  fireEvent.pointerDown(dialog, { clientX: 20, clientY: 20 });
  fireEvent.pointerUp(dialog, { clientX: 20, clientY: 20 });
  expect(onClose).toHaveBeenCalledTimes(1);
});
