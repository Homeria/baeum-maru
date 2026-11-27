import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test } from "vitest";
import { Checkbox } from "./Checkbox";
import { Radio } from "./Radio";

test("Checkbox는 라벨 클릭으로 켜고 끌 수 있고 설명을 연결한다", async () => {
  const user = userEvent.setup();

  render(<Checkbox label="기초생활수급자" description="우선 배정 대상" />);

  const box = screen.getByRole("checkbox", { name: "기초생활수급자" });
  expect(box).not.toBeChecked();
  expect(box).toHaveAccessibleDescription("우선 배정 대상");

  await user.click(screen.getByText("기초생활수급자"));
  expect(box).toBeChecked();

  await user.click(screen.getByText("기초생활수급자"));
  expect(box).not.toBeChecked();
});

test("Radio는 같은 name 묶음에서 하나만 선택된다", async () => {
  const user = userEvent.setup();

  render(
    <fieldset>
      <legend>실행 모드</legend>
      <Radio
        name="mode"
        value="host"
        label="호스트 PC"
        description="데이터를 이 PC에 저장합니다"
      />
      <Radio
        name="mode"
        value="remote"
        label="접속 PC"
        description="호스트 PC에 연결합니다"
      />
    </fieldset>,
  );

  const host = screen.getByRole("radio", { name: "호스트 PC" });
  const remote = screen.getByRole("radio", { name: "접속 PC" });
  expect(host).toHaveAccessibleDescription("데이터를 이 PC에 저장합니다");

  await user.click(screen.getByText("호스트 PC"));
  expect(host).toBeChecked();
  expect(remote).not.toBeChecked();

  await user.click(screen.getByText("접속 PC"));
  expect(host).not.toBeChecked();
  expect(remote).toBeChecked();
});
