import { render, screen, within } from "@testing-library/react";
import { expect, test } from "vitest";
import { SplitAuthLayout } from "./SplitAuthLayout";

test("좌측 안내 패널, 진행 단계(aria-current=step), 우측 메인 카드를 렌더링한다", () => {
  render(
    <SplitAuthLayout
      panelTitle="처음 설정"
      panelDescription="관리자 계정을 만들고 복구 코드를 보관합니다."
      steps={[
        { label: "관리자 계정 만들기", status: "done" },
        { label: "복구 코드 보관", status: "current" },
        { label: "기초 정보 미리 넣기", status: "upcoming" },
      ]}
      cardTitle="복구 코드를 적어 두세요"
      cardSubtitle="PIN을 잊었을 때 이 코드로 다시 정할 수 있습니다."
      cardFooter={<p>안전한 곳에 보관해 주세요.</p>}
    >
      <p>본문 영역</p>
    </SplitAuthLayout>,
  );

  expect(screen.getByRole("complementary", { name: "안내" })).toHaveTextContent(
    "배움마루",
  );
  const progress = screen.getByRole("navigation", { name: "진행 단계" });
  const items = within(progress).getAllByRole("listitem");
  expect(items).toHaveLength(3);
  expect(items[0]).not.toHaveAttribute("aria-current");
  expect(items[1]).toHaveAttribute("aria-current", "step");
  expect(items[2]).not.toHaveAttribute("aria-current");

  const main = screen.getByRole("main");
  expect(
    within(main).getByRole("heading", {
      level: 1,
      name: "복구 코드를 적어 두세요",
    }),
  ).toBeInTheDocument();
  expect(within(main).getByText("본문 영역")).toBeInTheDocument();
  expect(
    within(main).getByText("안전한 곳에 보관해 주세요."),
  ).toBeInTheDocument();
});
