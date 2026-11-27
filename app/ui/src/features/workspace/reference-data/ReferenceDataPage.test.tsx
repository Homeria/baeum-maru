import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import { setSession, type Session } from "../../../api/session";
import {
  createPriorityReason,
  deletePriorityReason,
  listPriorityReasons,
  updatePriorityReason,
} from "../priority-reasons/api";
import { createRoom, deleteRoom, listRooms, updateRoom } from "../rooms/api";
import {
  createSubject,
  deleteSubject,
  listSubjects,
  updateSubject,
} from "../subjects/api";
import { ReferenceDataPage } from "./ReferenceDataPage";

vi.mock("../subjects/api", () => ({
  listSubjects: vi.fn(),
  createSubject: vi.fn(),
  updateSubject: vi.fn(),
  deleteSubject: vi.fn(),
}));

vi.mock("../rooms/api", () => ({
  listRooms: vi.fn(),
  createRoom: vi.fn(),
  updateRoom: vi.fn(),
  deleteRoom: vi.fn(),
}));

vi.mock("../priority-reasons/api", () => ({
  listPriorityReasons: vi.fn(),
  createPriorityReason: vi.fn(),
  updatePriorityReason: vi.fn(),
  deletePriorityReason: vi.fn(),
}));

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
      <ReferenceDataPage />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(listSubjects).mockResolvedValue({
    items: [{ id: 1, name: "서예", category: "문화" }],
  });
  vi.mocked(createSubject).mockResolvedValue({ id: 2 });
  vi.mocked(updateSubject).mockResolvedValue({
    id: 1,
    name: "서예",
    category: "문화",
  });
  vi.mocked(deleteSubject).mockResolvedValue(undefined);

  vi.mocked(listRooms).mockResolvedValue({
    items: [
      { id: 10, building: "본관", floor: 2, name: "201호" },
      { id: 11, building: "별관", floor: 1, name: "101호" },
    ],
  });
  vi.mocked(createRoom).mockResolvedValue({ id: 12 });
  vi.mocked(updateRoom).mockResolvedValue({
    id: 10,
    building: "본관",
    floor: 2,
    name: "201호",
  });
  vi.mocked(deleteRoom).mockResolvedValue(undefined);

  vi.mocked(listPriorityReasons).mockResolvedValue({
    items: [{ id: 20, name: "지난 학기 반장", method: "guaranteed" }],
  });
  vi.mocked(createPriorityReason).mockResolvedValue({ id: 21 });
  vi.mocked(updatePriorityReason).mockResolvedValue({
    id: 20,
    name: "지난 학기 반장",
    method: "guaranteed",
  });
  vi.mocked(deletePriorityReason).mockResolvedValue(undefined);
});

test("과목 탭에서 과목 등록 서랍과 닫기 확인, 등록 알림이 동작한다", async () => {
  const user = userEvent.setup();
  renderPage("staff");

  expect(await screen.findByText("서예")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "과목 등록" }));

  const drawer = screen.getByRole("dialog", { name: "새 과목 등록" });
  await user.type(within(drawer).getByLabelText("이름"), "요가");
  await user.click(within(drawer).getByRole("button", { name: "그만두기" }));
  expect(
    screen.getByRole("dialog", { name: "저장하지 않은 내용이 있습니다" }),
  ).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "계속 편집" }));

  await user.click(within(drawer).getByRole("button", { name: "등록" }));
  expect(createSubject).toHaveBeenCalledWith({
    name: "요가",
    category: null,
  });
  expect(await screen.findByRole("status")).toHaveTextContent(
    "등록했습니다: 요가",
  );
});

test("강의실 탭에서 여러 건물 열을 보여 주고 서랍에서 삭제 확인 뒤 지운다", async () => {
  const user = userEvent.setup();
  renderPage("admin");

  await user.click(screen.getByRole("tab", { name: "강의실" }));
  expect(await screen.findByText("본관")).toBeInTheDocument();
  expect(screen.getByText("별관")).toBeInTheDocument();

  const editButtons = screen.getAllByRole("button", { name: "고치기" });
  await user.click(editButtons[0]);

  const drawer = screen.getByRole("dialog", { name: "강의실 고치기" });
  await user.click(within(drawer).getByRole("button", { name: "지우기" }));

  const confirm = screen.getByRole("dialog", {
    name: "201호 강의실을 지울까요?",
  });
  await user.click(within(confirm).getByRole("button", { name: "지우기" }));

  expect(deleteRoom).toHaveBeenCalledWith(10, expect.anything());
  expect(await screen.findByRole("status")).toHaveTextContent(
    "지웠습니다: 201호",
  );
});

test("우선 배정 사유 탭은 일반 직원에게는 안내만 띄우고 관리자만 등록·고치기가 된다", async () => {
  const user = userEvent.setup();
  const { unmount } = renderPage("staff");

  await user.click(screen.getByRole("tab", { name: "우선 배정 사유" }));
  expect(await screen.findByText("지난 학기 반장")).toBeInTheDocument();
  expect(
    screen.getByText("우선 배정 사유는 관리자만 등록하거나 바꿀 수 있습니다."),
  ).toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "사유 등록" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "고치기" }),
  ).not.toBeInTheDocument();

  unmount();
  renderPage("admin");
  await user.click(screen.getByRole("tab", { name: "우선 배정 사유" }));
  expect(await screen.findByText("지난 학기 반장")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "사유 등록" }));
  const drawer = screen.getByRole("dialog", { name: "새 우선 배정 사유" });
  await user.type(within(drawer).getByLabelText("이름"), "기초생활수급자");
  await user.click(within(drawer).getByRole("button", { name: "등록" }));

  expect(createPriorityReason).toHaveBeenCalledWith({
    name: "기초생활수급자",
    method: "guaranteed",
  });
});
