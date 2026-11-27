import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { SelectField } from "./SelectField";
import { TextField } from "./TextField";

test("TextField는 라벨·필수 꼬리표·도움말을 입력 칸과 연결한다", () => {
  render(
    <TextField
      label="회원 이름"
      required
      hint="실명 2~20자"
      placeholder="홍길동"
    />,
  );

  const input = screen.getByLabelText("회원 이름");
  expect(input).toBeRequired();
  expect(input).not.toHaveAttribute("aria-invalid");
  expect(input).toHaveAccessibleDescription("실명 2~20자");
  expect(screen.getByText("필수")).toBeInTheDocument();
});

test("TextField에 오류가 있으면 aria-invalid와 오류 알림을 연결한다", () => {
  render(
    <TextField
      label="연락처"
      hint="숫자만 입력해도 됩니다"
      error="연락처 형식이 올바르지 않습니다"
    />,
  );

  const input = screen.getByLabelText("연락처");
  expect(input).toHaveAttribute("aria-invalid", "true");
  expect(input).toHaveAccessibleDescription(
    "숫자만 입력해도 됩니다 연락처 형식이 올바르지 않습니다",
  );
  expect(screen.getByRole("alert")).toHaveTextContent(
    "연락처 형식이 올바르지 않습니다",
  );
});

test("SelectField는 라벨로 선택하고 오류 상태를 표시한다", async () => {
  const user = userEvent.setup();
  const onChange = vi.fn();

  render(
    <SelectField
      label="강의실"
      required
      error="강의실을 골라 주세요"
      defaultValue=""
      onChange={onChange}
    >
      <option value="">선택하세요</option>
      <option value="1">101호</option>
      <option value="2">202호</option>
    </SelectField>,
  );

  const select = screen.getByLabelText("강의실");
  expect(select).toBeRequired();
  expect(select).toHaveAttribute("aria-invalid", "true");
  expect(select).toHaveAccessibleDescription("강의실을 골라 주세요");

  await user.selectOptions(select, "2");
  expect(onChange).toHaveBeenCalled();
  expect(select).toHaveValue("2");
});
