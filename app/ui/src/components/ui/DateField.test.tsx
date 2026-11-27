import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, test, vi } from "vitest";
import { DateField } from "./DateField";
import { formatDateInput } from "./dateInput";

test("formatDateInput은 숫자 8자리와 구분자 날짜를 YYYY-MM-DD로 정리한다", () => {
  expect(formatDateInput("1955")).toBe("1955");
  expect(formatDateInput("19550")).toBe("1955-0");
  expect(formatDateInput("195503")).toBe("1955-03");
  expect(formatDateInput("19550315")).toBe("1955-03-15");
  expect(formatDateInput("1955031599")).toBe("1955-03-15");
  expect(formatDateInput("1955-")).toBe("1955");
  expect(formatDateInput("1955.3.5")).toBe("1955-03-05");
  expect(formatDateInput("1955/03/15")).toBe("1955-03-15");
  expect(formatDateInput("1955년 3월 15일")).toBe("1955-03-15");
});

function ControlledDateField() {
  const [value, setValue] = useState("");
  return <DateField label="생년월일" value={value} onChange={setValue} />;
}

test("DateField에서 숫자 8자리를 치면 자동 하이픈이 붙고 백스페이스로 지워진다", async () => {
  const user = userEvent.setup();
  render(<ControlledDateField />);

  const input = screen.getByLabelText("생년월일");
  await user.type(input, "19550315");
  expect(input).toHaveValue("1955-03-15");

  await user.type(input, "{Backspace}{Backspace}{Backspace}{Backspace}");
  expect(input).toHaveValue("1955");

  await user.clear(input);
  fireEvent.change(input, { target: { value: "1948.7.9" } });
  expect(input).toHaveValue("1948-07-09");
});

test("showPicker를 지원하는 환경에서는 달력 아이콘 버튼으로 달력을 연다", async () => {
  const user = userEvent.setup();
  const original = HTMLInputElement.prototype.showPicker;
  const showPicker = vi.fn();
  Object.defineProperty(HTMLInputElement.prototype, "showPicker", {
    configurable: true,
    value: showPicker,
  });

  try {
    render(<ControlledDateField />);
    await user.click(
      screen.getByRole("button", { name: "생년월일 달력에서 고르기" }),
    );
    expect(showPicker).toHaveBeenCalledTimes(1);
  } finally {
    if (original === undefined) {
      delete (
        HTMLInputElement.prototype as { showPicker?: (() => void) | undefined }
      ).showPicker;
    } else {
      Object.defineProperty(HTMLInputElement.prototype, "showPicker", {
        configurable: true,
        value: original,
      });
    }
  }
});
