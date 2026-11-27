import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import type { CourseLotteryResponse } from "../../../api/generated/CourseLotteryResponse";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { TermResponse } from "../../../api/generated/TermResponse";
import {
  listApplicationCounts,
  listCourses,
  listSelections,
} from "../courses/api";
import { listTerms } from "../terms/api";
import { closeSelection, getLottery, runLottery } from "./api";
import { SelectionPage } from "./SelectionPage";

vi.mock("./api", () => ({
  getLottery: vi.fn(),
  runLottery: vi.fn(),
  closeSelection: vi.fn(),
}));

vi.mock("../terms/api", () => ({
  listTerms: vi.fn(),
}));

vi.mock("../courses/api", () => ({
  listCourses: vi.fn(),
  listApplicationCounts: vi.fn(),
  listSelections: vi.fn(),
}));

const SAMPLE_TERM: TermResponse = {
  id: 1,
  name: "2026년 2학기",
  startsOn: "2026-09-01",
  endsOn: "2026-12-20",
};

const SAMPLE_COURSE: CourseResponse = {
  id: 100,
  termId: 1,
  subjectId: 1,
  subjectName: "서예",
  level: "초급",
  section: "1반",
  capacity: 2,
  schedules: [
    {
      weekday: 1,
      start: "10:00",
      end: "11:00",
      room: { id: 1, building: "본관", name: "201호" },
    },
  ],
};

const BEFORE_DRAW: CourseLotteryResponse = {
  capacity: 2,
  stage: "not_drawn",
  selectedCount: 0,
  canDraw: true,
  entrants: [
    {
      applicationId: 1,
      memberName: "김반장",
      memberNo: "M001",
      guaranteed: true,
    },
    {
      applicationId: 2,
      memberName: "이신청",
      memberNo: "M002",
      guaranteed: false,
    },
    {
      applicationId: 3,
      memberName: "박대기",
      memberNo: "M003",
      guaranteed: false,
    },
  ],
};

const AFTER_DRAW: CourseLotteryResponse = {
  ...BEFORE_DRAW,
  stage: "drawn",
  selectedCount: 2,
};

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <SelectionPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(listTerms).mockResolvedValue({ items: [SAMPLE_TERM] });
  vi.mocked(listCourses).mockResolvedValue({ items: [SAMPLE_COURSE] });
  vi.mocked(listApplicationCounts).mockResolvedValue({
    items: [{ courseId: 100, count: 3 }],
  });
  vi.mocked(listSelections).mockResolvedValue({ items: [] });
  vi.mocked(getLottery).mockResolvedValue(BEFORE_DRAW);
  vi.mocked(runLottery).mockResolvedValue(AFTER_DRAW);
  vi.mocked(closeSelection).mockResolvedValue(undefined);
});

test("강좌를 고르면 강좌명 제목·우선 배정 안내·3칸 목록이 보이고 시작과 멈춤으로 추첨한다", async () => {
  const user = userEvent.setup();
  renderPage();

  const select = await screen.findByLabelText("강좌");
  await user.selectOptions(select, "100");

  expect(
    await screen.findByRole("heading", { level: 2, name: "서예 초급 1반" }),
  ).toBeInTheDocument();

  const applicants = screen.getByRole("region", { name: "신청자" });
  expect(
    within(applicants).getByText("우선 배정 1명은 추첨 없이 당첨자에 있습니다"),
  ).toBeInTheDocument();

  const winners = screen.getByRole("region", { name: "당첨자" });
  expect(within(winners).getByText("김반장 (M001)")).toBeInTheDocument();
  expect(within(winners).getByText("우선")).toBeInTheDocument();
  expect(winners.querySelector("ol")).toBeNull();

  await user.click(screen.getByRole("button", { name: "시작" }));
  const waitlistedDuring = screen.getByRole("region", { name: "대기자" });
  expect(applicants).not.toHaveAttribute("data-shuffling", "true");
  expect(winners).toHaveAttribute("data-shuffling", "true");
  expect(waitlistedDuring).toHaveAttribute("data-shuffling", "true");
  expect(within(winners).getByText("섞는 중")).toBeInTheDocument();
  expect(within(waitlistedDuring).getByText("섞는 중")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "멈춤" }));

  expect(runLottery).toHaveBeenCalledWith(100);

  const waitlisted = screen.getByRole("region", { name: "대기자" });
  expect(waitlisted.querySelector("ol")).not.toBeNull();
  expect(within(waitlisted).getByText("박대기 (M003)")).toBeInTheDocument();
});

test("추첨 뒤 선발 마감과 다시 추첨 확인 대화상자가 동작한다", async () => {
  const user = userEvent.setup();
  vi.mocked(getLottery).mockResolvedValue(AFTER_DRAW);
  renderPage();

  await user.selectOptions(await screen.findByLabelText("강좌"), "100");
  await user.click(await screen.findByRole("button", { name: "선발 마감" }));

  const dialog = screen.getByRole("dialog", {
    name: "서예 초급 1반의 선발을 마감할까요?",
  });
  expect(
    within(dialog).getByRole("button", { name: "그만두기" }),
  ).toHaveFocus();

  await user.click(within(dialog).getByRole("button", { name: "선발 마감" }));
  expect(closeSelection).toHaveBeenCalledWith(100);
});

test("추첨 권한이 없으면 버튼을 숨기고 결과 보기 안내를 띄운다", async () => {
  const user = userEvent.setup();
  vi.mocked(getLottery).mockResolvedValue({
    ...BEFORE_DRAW,
    canDraw: false,
  });
  renderPage();

  await user.selectOptions(await screen.findByLabelText("강좌"), "100");
  expect(
    await screen.findByText(
      "추첨과 선발 마감은 호스트 PC에서 관리자·직원이 합니다. 이 화면에서는 결과만 봅니다.",
    ),
  ).toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "시작" }),
  ).not.toBeInTheDocument();
});
