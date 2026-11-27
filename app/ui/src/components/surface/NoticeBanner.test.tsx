import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { NoticeBanner } from "./NoticeBanner";

test("기본 안내 배너는 status 역할로 빈 상태 문구를 보여 준다", () => {
  render(<NoticeBanner message="등록된 회원이 없습니다." />);

  expect(screen.getByRole("status")).toHaveTextContent(
    "등록된 회원이 없습니다.",
  );
});

test("오류 배너는 alert 역할로 표시되고 다시 해 보기 버튼을 실행한다", async () => {
  const user = userEvent.setup();
  const onRetry = vi.fn();

  render(
    <NoticeBanner
      tone="error"
      title="불러오지 못했습니다"
      message="호스트 연결을 확인해 주세요."
      actionLabel="다시 해 보기"
      onAction={onRetry}
    />,
  );

  expect(screen.getByRole("alert")).toHaveTextContent("불러오지 못했습니다");
  await user.click(screen.getByRole("button", { name: "다시 해 보기" }));
  expect(onRetry).toHaveBeenCalledTimes(1);
});
