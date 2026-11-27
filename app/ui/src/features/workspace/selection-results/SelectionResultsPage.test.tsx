import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { CourseResultResponse } from "../../../api/generated/CourseResultResponse";
import type { TermResponse } from "../../../api/generated/TermResponse";
import { saveFile } from "../../../api/saveFile";
import { setSession, type Session } from "../../../api/session";
import { listCourses } from "../courses/api";
import { listTerms } from "../terms/api";
import {
  correctWithdrawal,
  getCourseResult,
  listClosedCourses,
  recordContact,
  withdraw,
} from "./api";
import { exportRoster } from "./rostersApi";
import { SelectionResultsPage } from "./SelectionResultsPage";

vi.mock("./api", () => ({
  listClosedCourses: vi.fn(),
  getCourseResult: vi.fn(),
  withdraw: vi.fn(),
  recordContact: vi.fn(),
  correctWithdrawal: vi.fn(),
}));

vi.mock("./rostersApi", () => ({
  exportRoster: vi.fn(),
}));

vi.mock("../../../api/saveFile", () => ({
  saveFile: vi.fn(),
}));

vi.mock("../terms/api", () => ({
  listTerms: vi.fn(),
}));

vi.mock("../courses/api", () => ({
  listCourses: vi.fn(),
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
  schedules: [],
};

const SAMPLE_RESULT: CourseResultResponse = {
  capacity: 2,
  openSeats: 1,
  canRecord: true,
  canCorrect: true,
  selected: [
    {
      member: { applicationId: 1, memberName: "김당첨", memberNo: "M001" },
      guaranteed: true,
      waitlistNumber: null,
    },
  ],
  waitlisted: [
    {
      member: { applicationId: 2, memberName: "이대기", memberNo: "M002" },
      phoneNumber: "01022223333",
      waitlistNumber: 1,
      lastContact: null,
      contactTurn: true,
    },
  ],
  withdrawn: [
    {
      member: { applicationId: 3, memberName: "박포기", memberNo: "M003" },
      withdrawnFrom: "selected",
      waitlistNumber: null,
    },
  ],
};

function renderPage(role: Session["staff"]["role"] = "admin") {
  setSession({
    token: "tok",
    staff: { id: 1, name: "직원1", role },
  });
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <SelectionResultsPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  vi.mocked(listTerms).mockResolvedValue({ items: [SAMPLE_TERM] });
  vi.mocked(listCourses).mockResolvedValue({ items: [SAMPLE_COURSE] });
  vi.mocked(listClosedCourses).mockResolvedValue({
    items: [
      {
        courseId: 100,
        capacity: 2,
        selected: 1,
        waitlisted: 1,
        openSeats: 1,
        nextAdditional: { kind: "waitlisted", waitlistNumber: 2 },
      },
    ],
  });
  vi.mocked(getCourseResult).mockResolvedValue(SAMPLE_RESULT);
  vi.mocked(withdraw).mockResolvedValue(undefined);
  vi.mocked(recordContact).mockResolvedValue(undefined);
  vi.mocked(correctWithdrawal).mockResolvedValue(undefined);
  vi.mocked(exportRoster).mockResolvedValue(new Uint8Array([1, 2, 3]));
  vi.mocked(saveFile).mockResolvedValue("/tmp/roster.xlsx");
});

test("마감 강좌를 고르면 연락할 차례를 강조하고 대기자 연락·포기 창에서 수락과 본인 포기를 기록한다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  await user.click(
    await screen.findByRole("button", { name: /서예 초급 1반/ }),
  );

  const waitlisted = await screen.findByRole("region", { name: "대기자" });
  expect(within(waitlisted).getByText("연락할 차례")).toBeInTheDocument();
  expect(within(waitlisted).getByText("010-2222-3333")).toBeInTheDocument();
  expect(
    within(waitlisted).queryByRole("button", { name: "포기" }),
  ).not.toBeInTheDocument();

  await user.click(
    within(waitlisted).getByRole("button", { name: "연락·포기" }),
  );
  const dialog = screen.getByRole("dialog", { name: "대기자 연락·포기 처리" });
  expect(within(dialog).getByText(/010-2222-3333/)).toBeInTheDocument();
  expect(
    within(dialog).getByRole("button", { name: "그만두기" }),
  ).toHaveFocus();

  await user.click(within(dialog).getByRole("button", { name: "수락 (당첨)" }));
  expect(recordContact).toHaveBeenCalledWith(2, "accepted");
  expect(
    await screen.findByText("이대기 님이 수락해 당첨으로 바꿨습니다."),
  ).toBeInTheDocument();

  await user.click(
    within(waitlisted).getByRole("button", { name: "연락·포기" }),
  );
  const withdrawDialog = screen.getByRole("dialog", {
    name: "대기자 연락·포기 처리",
  });
  await user.type(
    within(withdrawDialog).getByLabelText("사유 (선택)"),
    "본인 전화",
  );
  await user.click(
    within(withdrawDialog).getByRole("button", { name: "포기로 바꾸기" }),
  );
  expect(withdraw).toHaveBeenCalledWith(2, "본인 전화");
});

test("포기와 관리자 전용 바로잡기 대화상자가 사유 검증과 함께 동작한다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  await user.click(
    await screen.findByRole("button", { name: /서예 초급 1반/ }),
  );

  const withdrawn = await screen.findByRole("region", { name: "포기" });
  await user.click(within(withdrawn).getByRole("button", { name: "바로잡기" }));

  const dialog = screen.getByRole("dialog", {
    name: "잘못 처리한 포기를 바로잡을까요?",
  });
  await user.click(within(dialog).getByRole("button", { name: "바로잡기" }));
  expect(correctWithdrawal).not.toHaveBeenCalled();
  expect(
    within(dialog).getByText("바로잡는 사유를 적어 주세요"),
  ).toBeInTheDocument();

  await user.type(within(dialog).getByLabelText("사유"), "잘못 누름");
  await user.click(within(dialog).getByRole("button", { name: "바로잡기" }));
  expect(correctWithdrawal).toHaveBeenCalledWith(3, "잘못 누름");
});

test("결과 명단 내보내기 창은 조회 전용에게 원본 명단을 숨기고 파일을 저장한다", async () => {
  const user = userEvent.setup();
  renderPage("read_only");

  await user.click(
    await screen.findByRole("button", { name: "결과 명단 내보내기" }),
  );
  const dialog = screen.getByRole("dialog", { name: "결과 명단 내보내기" });
  expect(
    within(dialog).queryByRole("radio", { name: "원본 명단" }),
  ).not.toBeInTheDocument();

  await user.click(within(dialog).getByRole("button", { name: "내보내기" }));
  expect(exportRoster).toHaveBeenCalledTimes(1);
  expect(saveFile).toHaveBeenCalledTimes(1);
  expect(
    await screen.findByText("저장했습니다: /tmp/roster.xlsx"),
  ).toBeInTheDocument();
});

test("기록·바로잡기 권한이 없으면 포기·연락·포기·바로잡기 버튼을 숨긴다", async () => {
  const user = userEvent.setup();
  vi.mocked(getCourseResult).mockResolvedValue({
    ...SAMPLE_RESULT,
    canRecord: false,
    canCorrect: false,
  });
  renderPage("read_only");

  await user.click(
    await screen.findByRole("button", { name: /서예 초급 1반/ }),
  );
  expect(await screen.findByText("김당첨 (M001)")).toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "연락·포기" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "포기" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "바로잡기" }),
  ).not.toBeInTheDocument();
});
