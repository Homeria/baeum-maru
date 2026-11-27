import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import type { TermResponse } from "../../../api/generated/TermResponse";
import { setSession, type Session } from "../../../api/session";
import { listRooms } from "../rooms/api";
import { listSubjects } from "../subjects/api";
import { createTerm, deleteTerm, listTerms, updateTerm } from "../terms/api";
import { createCourse, deleteCourse, listCourses, updateCourse } from "./api";
import { CoursesPage } from "./CoursesPage";

vi.mock("./api", () => ({
  listCourses: vi.fn(),
  createCourse: vi.fn(),
  updateCourse: vi.fn(),
  deleteCourse: vi.fn(),
}));

vi.mock("../terms/api", () => ({
  listTerms: vi.fn(),
  createTerm: vi.fn(),
  updateTerm: vi.fn(),
  deleteTerm: vi.fn(),
}));

vi.mock("../subjects/api", () => ({
  listSubjects: vi.fn(),
}));

vi.mock("../rooms/api", () => ({
  listRooms: vi.fn(),
}));

const SAMPLE_TERM: TermResponse = {
  id: 1,
  name: "2026년 2학기",
  startsOn: "2026-09-01",
  endsOn: "2026-12-20",
};

const SAMPLE_SUBJECT: SubjectResponse = {
  id: 10,
  name: "서예",
  category: "문화",
};

const SAMPLE_ROOM: RoomResponse = {
  id: 20,
  building: "본관",
  floor: 2,
  name: "201호",
};

const SAMPLE_COURSE: CourseResponse = {
  id: 100,
  termId: 1,
  subjectId: 10,
  subjectName: "서예",
  level: "초급",
  section: "1반",
  capacity: 20,
  schedules: [
    {
      weekday: 1,
      start: "10:00",
      end: "11:00",
      room: { id: 20, building: "본관", name: "201호" },
    },
    {
      weekday: 3,
      start: "14:00",
      end: "15:00",
      room: { id: 20, building: "본관", name: "201호" },
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
      <CoursesPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(listTerms).mockResolvedValue({ items: [SAMPLE_TERM] });
  vi.mocked(listCourses).mockResolvedValue({ items: [SAMPLE_COURSE] });
  vi.mocked(listSubjects).mockResolvedValue({ items: [SAMPLE_SUBJECT] });
  vi.mocked(listRooms).mockResolvedValue({ items: [SAMPLE_ROOM] });
  vi.mocked(createCourse).mockResolvedValue({ id: 101 });
  vi.mocked(updateCourse).mockResolvedValue(SAMPLE_COURSE);
  vi.mocked(deleteCourse).mockResolvedValue(undefined);
  vi.mocked(createTerm).mockResolvedValue({ id: 2 });
  vi.mocked(updateTerm).mockResolvedValue(SAMPLE_TERM);
  vi.mocked(deleteTerm).mockResolvedValue(undefined);
});

test("여러 시간대 강좌를 줄바꿈해 보여 주고 강좌 등록 서랍과 닫기 확인이 동작한다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  expect(await screen.findByText("월 10:00~11:00")).toBeInTheDocument();
  expect(screen.getByText("수 14:00~15:00")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "강좌 등록" }));
  const drawer = screen.getByRole("dialog", { name: "새 강좌 등록" });

  await user.selectOptions(within(drawer).getByLabelText("과목"), "10");
  await user.type(within(drawer).getByLabelText("정원"), "15");
  await user.type(within(drawer).getByLabelText("1번째 시작 시각"), "09:00");
  await user.type(within(drawer).getByLabelText("1번째 종료 시각"), "10:00");

  await user.click(within(drawer).getByRole("button", { name: "그만두기" }));
  expect(
    screen.getByRole("dialog", { name: "저장하지 않은 내용이 있습니다" }),
  ).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "계속 편집" }));

  await user.click(within(drawer).getByRole("button", { name: "등록" }));
  expect(createCourse).toHaveBeenCalledWith(1, {
    subjectId: 10,
    level: "",
    section: "",
    capacity: 15,
    schedules: [{ weekday: 1, start: "09:00", end: "10:00", roomId: null }],
  });
  expect(await screen.findByRole("status")).toHaveTextContent(
    "등록했습니다: 서예",
  );
});

test("학기 고치기 모달과 강좌 지우기 확인 대화상자가 동작한다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  expect(await screen.findByText("서예")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "학기 고치기" }));
  const termDialog = screen.getByRole("dialog", { name: "학기 고치기" });
  expect(within(termDialog).getByLabelText("이름")).toHaveFocus();
  await user.click(
    within(termDialog).getByRole("button", { name: "그만두기" }),
  );

  await user.click(screen.getByRole("button", { name: "고치기" }));
  const drawer = screen.getByRole("dialog", { name: "강좌 고치기" });
  await user.click(within(drawer).getByRole("button", { name: "지우기" }));

  const confirm = screen.getByRole("dialog", {
    name: "서예 초급 1반 강좌를 지울까요?",
  });
  expect(
    within(confirm).getByRole("button", { name: "그만두기" }),
  ).toHaveFocus();
  await user.click(within(confirm).getByRole("button", { name: "지우기" }));

  expect(deleteCourse).toHaveBeenCalledWith(100, expect.anything());
  expect(await screen.findByRole("status")).toHaveTextContent(
    "지웠습니다: 서예 초급 1반",
  );
});

test("조회 전용 역할에게는 학기·강좌 편집 버튼을 모두 숨긴다", async () => {
  const user = userEvent.setup();
  renderPage("read_only");

  expect(await screen.findByText("서예")).toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "학기 고치기" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "새 학기" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "강좌 등록" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "고치기" }),
  ).not.toBeInTheDocument();

  await user.click(screen.getByText("서예"));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});
