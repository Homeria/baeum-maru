import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import { HostError } from "../../../api/client";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { MemberResponse } from "../../../api/generated/MemberResponse";
import type { TermResponse } from "../../../api/generated/TermResponse";
import { setSession, type Session } from "../../../api/session";
import {
  listApplicationCounts,
  listCourses,
  listSelections,
} from "../courses/api";
import { getMember, searchMembers } from "../members/api";
import { listPriorityReasons } from "../priority-reasons/api";
import { listRooms } from "../rooms/api";
import { listClosedCourses } from "../selection-results/api";
import { listTerms } from "../terms/api";
import {
  deleteApplication,
  listApplications,
  setPriorityReasons,
  submitApplications,
} from "./api";
import { ApplicationsPage } from "./ApplicationsPage";

vi.mock("./api", () => ({
  listApplications: vi.fn(),
  submitApplications: vi.fn(),
  deleteApplication: vi.fn(),
  setPriorityReasons: vi.fn(),
}));

vi.mock("../terms/api", () => ({
  listTerms: vi.fn(),
}));

vi.mock("../courses/api", () => ({
  listCourses: vi.fn(),
  listApplicationCounts: vi.fn(),
  listSelections: vi.fn(),
}));

vi.mock("../members/api", () => ({
  searchMembers: vi.fn(),
  getMember: vi.fn(),
}));

vi.mock("../rooms/api", () => ({
  listRooms: vi.fn(),
}));

vi.mock("../priority-reasons/api", () => ({
  listPriorityReasons: vi.fn(),
}));

vi.mock("../selection-results/api", () => ({
  listClosedCourses: vi.fn(),
}));

const SAMPLE_TERM: TermResponse = {
  id: 1,
  name: "2026년 2학기",
  startsOn: "2026-09-01",
  endsOn: "2099-12-31",
};

const SAMPLE_MEMBER: MemberResponse = {
  id: 10,
  memberNo: "M001",
  name: "홍길동",
  phoneNumber: "01012345678",
  birthDate: "1955-03-15",
  gender: "male",
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

const SAMPLE_COURSES: CourseResponse[] = [
  {
    id: 100,
    termId: 1,
    subjectId: 1,
    subjectName: "서예",
    level: "초급",
    section: "1반",
    capacity: 20,
    schedules: [
      {
        weekday: 1,
        start: "10:00",
        end: "11:00",
        room: { id: 1, building: "본관", name: "201호" },
      },
      {
        weekday: 3,
        start: "14:00",
        end: "15:00",
        room: { id: 1, building: "본관", name: "201호" },
      },
    ],
  },
  {
    id: 101,
    termId: 1,
    subjectId: 2,
    subjectName: "요가",
    level: null,
    section: null,
    capacity: 15,
    schedules: [
      {
        weekday: 2,
        start: "10:00",
        end: "11:00",
        room: { id: 1, building: "본관", name: "201호" },
      },
    ],
  },
];

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
      <ApplicationsPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(listTerms).mockResolvedValue({ items: [SAMPLE_TERM] });
  vi.mocked(listCourses).mockResolvedValue({ items: SAMPLE_COURSES });
  vi.mocked(listApplicationCounts).mockResolvedValue({
    items: [
      { courseId: 100, count: 5 },
      { courseId: 101, count: 14 },
    ],
  });
  vi.mocked(listSelections).mockResolvedValue({
    items: [{ courseId: 101, stage: "closed" }],
  });
  vi.mocked(listClosedCourses).mockResolvedValue({
    items: [
      {
        courseId: 101,
        capacity: 15,
        selected: 14,
        waitlisted: 0,
        openSeats: 1,
        nextAdditional: { kind: "selected" },
      },
    ],
  });
  vi.mocked(listRooms).mockResolvedValue({
    items: [{ id: 1, building: "본관", floor: 2, name: "201호" }],
  });
  vi.mocked(listPriorityReasons).mockResolvedValue({
    items: [{ id: 7, name: "지난 학기 반장", method: "guaranteed" }],
  });
  vi.mocked(searchMembers).mockResolvedValue({
    items: [SAMPLE_MEMBER],
    total: 1,
    page: 1,
    pageSize: 20,
  });
  vi.mocked(getMember).mockResolvedValue(SAMPLE_MEMBER);
  vi.mocked(listApplications).mockResolvedValue({ items: [] });
  vi.mocked(submitApplications).mockResolvedValue({
    items: [{ id: 500, courseId: 100, additional: null }],
  });
  vi.mocked(deleteApplication).mockResolvedValue(undefined);
  vi.mocked(setPriorityReasons).mockResolvedValue({ lotteryCancelled: false });
});

test("회원을 고르고 다중 수업 시간·추가 신청 꼬리표를 확인한 뒤 우선 배정으로 신청한다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  expect(await screen.findByText("월 10:00~11:00")).toBeInTheDocument();
  expect(screen.getByText("수 14:00~15:00")).toBeInTheDocument();
  expect(screen.getByText("추가 신청: 바로 당첨")).toBeInTheDocument();

  await user.click(await screen.findByText("홍길동"));
  await user.click(screen.getByLabelText("서예 초급 1반 고르기"));

  const priorityGroup = screen.getByRole("group", {
    name: "서예 초급 1반 우선 배정",
  });
  await user.click(
    within(priorityGroup).getByRole("checkbox", { name: "지난 학기 반장" }),
  );
  await user.click(screen.getByRole("button", { name: "1건 신청하기" }));

  expect(submitApplications).toHaveBeenCalledWith(1, {
    memberId: 10,
    courseIds: [100],
    overrideReason: null,
    priorities: [{ courseId: 100, reasonIds: [7] }],
  });
  expect(await screen.findByText("1건 신청했습니다")).toBeInTheDocument();
});

test("신청 규칙 위반 시 예외 사유 대화상자가 열리고 사유를 적어 다시 신청한다", async () => {
  const user = userEvent.setup();
  vi.mocked(submitApplications)
    .mockRejectedValueOnce(
      new HostError(
        422,
        "규칙 위반",
        "RULES_BROKEN",
        null,
        [],
        [
          {
            courseId: 100,
            rule: "no_time_overlap",
            otherCourseId: 101,
            canOverride: true,
          },
        ],
      ),
    )
    .mockResolvedValueOnce({
      items: [{ id: 501, courseId: 100, additional: null }],
    });

  renderPage("staff");

  await user.click(await screen.findByText("홍길동"));
  await user.click(screen.getByLabelText("서예 초급 1반 고르기"));
  await user.click(screen.getByRole("button", { name: "1건 신청하기" }));

  const dialog = await screen.findByRole("dialog", {
    name: "신청 규칙을 어긴 강좌가 있습니다",
  });
  expect(within(dialog).getByLabelText("예외 사유")).toHaveFocus();

  await user.type(within(dialog).getByLabelText("예외 사유"), "강사 허락");
  await user.click(
    within(dialog).getByRole("button", { name: "사유 적고 신청" }),
  );

  expect(submitApplications).toHaveBeenLastCalledWith(1, {
    memberId: 10,
    courseIds: [100],
    overrideReason: "강사 허락",
    priorities: [],
  });
});

test("고른 회원의 신청을 지우기 대화상자에서 확인해 삭제한다", async () => {
  const user = userEvent.setup();
  vi.mocked(listApplications).mockResolvedValue({
    items: [
      {
        id: 500,
        courseId: 100,
        submittedAt: "2026-06-01T00:00:00Z",
        overriddenRules: [],
        priorityReasonIds: [],
      },
    ],
  });
  renderPage("admin");

  await user.click(await screen.findByText("홍길동"));
  await user.click(await screen.findByRole("button", { name: "지우기" }));

  const dialog = screen.getByRole("dialog", {
    name: "서예 초급 1반 신청을 지울까요?",
  });
  expect(
    within(dialog).getByRole("button", { name: "그만두기" }),
  ).toHaveFocus();
  await user.type(within(dialog).getByLabelText("사유 (선택)"), "단순 변심");
  await user.click(within(dialog).getByRole("button", { name: "지우기" }));

  expect(deleteApplication).toHaveBeenCalledWith(500, { reason: "단순 변심" });
});

test("조회 전용 역할에게는 신청 막대와 신청 지우기 버튼을 숨긴다", async () => {
  const user = userEvent.setup();
  vi.mocked(listApplications).mockResolvedValue({
    items: [
      {
        id: 500,
        courseId: 100,
        submittedAt: "2026-06-01T00:00:00Z",
        overriddenRules: [],
        priorityReasonIds: [],
      },
    ],
  });
  renderPage("read_only");

  await user.click(await screen.findByText("홍길동"));
  expect(
    await screen.findByText("홍길동(M001)의 신청 · 1건"),
  ).toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: /신청하기/ }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "지우기" }),
  ).not.toBeInTheDocument();
});
