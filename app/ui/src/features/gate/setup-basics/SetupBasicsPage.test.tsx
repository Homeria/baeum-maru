import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import { clearSession, setSession } from "../../../api/session";
import { StaffFlow } from "../../../app/StaffFlow";
import {
  createPriorityReason,
  deletePriorityReason,
  listPriorityReasons,
} from "../../workspace/priority-reasons/api";
import { createRoom, deleteRoom, listRooms } from "../../workspace/rooms/api";
import {
  createSubject,
  deleteSubject,
  listSubjects,
} from "../../workspace/subjects/api";
import { listLoginStaff, login } from "../auth/api";
import { recover } from "../auth/recoveryApi";
import { fetchSetupStatus, setup } from "../auth/setupApi";
import { useRunMode } from "../run-mode/useRunMode";
import { SetupBasicsPage } from "./SetupBasicsPage";

vi.mock("../../workspace/rooms/api", () => ({
  listRooms: vi.fn(),
  createRoom: vi.fn(),
  deleteRoom: vi.fn(),
}));

vi.mock("../../workspace/subjects/api", () => ({
  listSubjects: vi.fn(),
  createSubject: vi.fn(),
  deleteSubject: vi.fn(),
}));

vi.mock("../../workspace/priority-reasons/api", () => ({
  listPriorityReasons: vi.fn(),
  createPriorityReason: vi.fn(),
  deletePriorityReason: vi.fn(),
}));

vi.mock("../auth/setupApi", () => ({
  fetchSetupStatus: vi.fn(),
  setup: vi.fn(),
}));

vi.mock("../auth/api", () => ({
  listLoginStaff: vi.fn(),
  login: vi.fn(),
}));

vi.mock("../auth/recoveryApi", () => ({
  recover: vi.fn(),
}));

vi.mock("../run-mode/useRunMode", () => ({
  useRunMode: vi.fn(),
}));

vi.mock("../../../app/MainScreen", () => ({
  MainScreen: ({ session }: { session: { staff: { name: string } } }) => (
    <div data-testid="main-screen">메인 화면: {session.staff.name}</div>
  ),
}));

let roomStore: RoomResponse[] = [];
let subjectStore: SubjectResponse[] = [];
let reasonStore: PriorityReasonResponse[] = [];
let nextId = 1;

function renderWithQuery(ui: React.ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  clearSession();
  roomStore = [];
  subjectStore = [];
  reasonStore = [];
  nextId = 1;

  vi.mocked(useRunMode).mockReturnValue({
    data: { mode: "host", hostUrl: null },
  } as unknown as ReturnType<typeof useRunMode>);

  vi.mocked(listRooms).mockImplementation(async () => ({
    items: [...roomStore],
  }));
  vi.mocked(createRoom).mockImplementation(async (request) => {
    const id = nextId++;
    roomStore.push({
      id,
      name: request.name.trim(),
      building: request.building.trim(),
      floor: request.floor,
    });
    return { id };
  });
  vi.mocked(deleteRoom).mockImplementation(async (id) => {
    roomStore = roomStore.filter((room) => room.id !== id);
  });

  vi.mocked(listSubjects).mockImplementation(async () => ({
    items: [...subjectStore],
  }));
  vi.mocked(createSubject).mockImplementation(async (request) => {
    const id = nextId++;
    subjectStore.push({
      id,
      name: request.name.trim(),
      category: request.category ? request.category.trim() : null,
    });
    return { id };
  });
  vi.mocked(deleteSubject).mockImplementation(async (id) => {
    subjectStore = subjectStore.filter((subject) => subject.id !== id);
  });

  vi.mocked(listPriorityReasons).mockImplementation(async () => ({
    items: [...reasonStore],
  }));
  vi.mocked(createPriorityReason).mockImplementation(async (request) => {
    const id = nextId++;
    reasonStore.push({
      id,
      name: request.name.trim(),
      method: request.method,
    });
    return { id };
  });
  vi.mocked(deletePriorityReason).mockImplementation(async (id) => {
    reasonStore = reasonStore.filter((reason) => reason.id !== id);
  });
});

test("강의실 단계는 건물·층 마지막 값을 유지하며 연속 입력하고 빼기로 지운다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();

  renderWithQuery(<SetupBasicsPage onDone={onDone} />);

  const buildingInput = await screen.findByLabelText("건물");
  const floorSelect = screen.getByLabelText("층");
  const nameInput = screen.getByLabelText(/강의실 이름/);

  expect(buildingInput).toHaveValue("본관");
  await user.selectOptions(floorSelect, "2");
  await user.type(nameInput, "1강의실");
  await user.click(screen.getByRole("button", { name: "더하기" }));

  expect(await screen.findByText("1강의실")).toBeInTheDocument();
  expect(buildingInput).toHaveValue("본관");
  expect(floorSelect).toHaveValue("2");
  expect(nameInput).toHaveValue("");
  expect(nameInput).toHaveFocus();

  await user.click(screen.getByRole("button", { name: "1강의실 빼기" }));
  expect(screen.queryByText("1강의실")).not.toBeInTheDocument();
});

test("과목·우선 배정 단계는 지난 학기 반장 추천 체크박스와 직접 추가, 과목 연속 입력을 처리한다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();

  renderWithQuery(<SetupBasicsPage onDone={onDone} />);

  await user.click(
    screen.getByRole("button", { name: "다음: 과목 · 우선 배정" }),
  );

  // 비민감 항목인 "지난 학기 반장" 1개만 추천 체크박스로 두고 민감 사유는 기본 추천하지 않는다
  const leaderCheckbox = await screen.findByRole("checkbox", {
    name: "지난 학기 반장",
  });
  expect(screen.getAllByRole("checkbox")).toHaveLength(1);
  expect(
    screen.getByText(/기관 개인정보 안내·동의 절차를 확인한 뒤 직접 추가/),
  ).toBeInTheDocument();

  await user.click(leaderCheckbox);
  expect(createPriorityReason).toHaveBeenCalledWith({
    name: "지난 학기 반장",
    method: "guaranteed",
  });
  expect(
    await screen.findByRole("checkbox", { name: "지난 학기 반장" }),
  ).toBeChecked();

  await user.click(leaderCheckbox);
  expect(deletePriorityReason).toHaveBeenCalledTimes(1);

  // 직접 사유 추가 및 빼기
  await user.type(screen.getByLabelText("사유 이름"), "다자녀");
  const addButtons = screen.getAllByRole("button", { name: "더하기" });
  await user.click(addButtons[1]);
  expect(await screen.findByText("다자녀")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "다자녀 빼기" }));
  expect(screen.queryByText("다자녀")).not.toBeInTheDocument();

  // 과목 분류 유지 + 추가
  const categoryInput = screen.getByLabelText("분류");
  const subjectNameInput = screen.getByLabelText(/과목 이름/);
  await user.type(categoryInput, "서예·미술");
  await user.type(subjectNameInput, "서예");
  await user.click(addButtons[0]);

  expect(await screen.findByText("서예")).toBeInTheDocument();
  expect(categoryInput).toHaveValue("서예·미술");
  expect(subjectNameInput).toHaveValue("");

  await user.click(
    screen.getByRole("button", { name: "설정 마치고 시작하기" }),
  );
  expect(onDone).toHaveBeenCalledTimes(1);
});

test("강의실 단계에서 나중에 하기를 누르면 즉시 마친다", async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();

  renderWithQuery(<SetupBasicsPage onDone={onDone} />);
  await user.click(screen.getByRole("button", { name: "나중에 하기" }));
  expect(onDone).toHaveBeenCalledTimes(1);
});

test("StaffFlow는 처음 설정 1→2→3단계를 거쳐 메인 화면으로 바로 진입한다", async () => {
  const user = userEvent.setup();
  let setupDone = false;
  vi.mocked(fetchSetupStatus).mockImplementation(async () => ({
    needed: !setupDone,
  }));
  vi.mocked(setup).mockImplementation(async () => {
    setupDone = true;
    return { recoveryCode: "K7QF-M2XA-9TRE-HW4C" };
  });
  vi.mocked(listLoginStaff).mockResolvedValue({
    items: [{ id: 1, name: "관리자1", role: "admin" }],
  });
  vi.mocked(login).mockImplementation(async () => {
    const response = {
      token: "tok",
      staff: { id: 1, name: "관리자1", role: "admin" as const },
    };
    setSession({ token: response.token, staff: response.staff });
    return response;
  });

  renderWithQuery(<StaffFlow />);

  // 1단계: 관리자 계정 만들기
  await user.type(await screen.findByLabelText("관리자 이름"), "관리자1");
  await user.type(screen.getByLabelText("PIN"), "123456");
  await user.type(screen.getByLabelText("PIN 한 번 더"), "123456");
  await user.click(screen.getByRole("button", { name: "관리자 만들기" }));

  // 2단계: 복구 코드 보관
  expect(await screen.findByText("K7QF-M2XA-9TRE-HW4C")).toBeInTheDocument();
  await user.click(
    screen.getByRole("checkbox", { name: "인쇄하거나 적어 두었습니다" }),
  );
  await user.click(screen.getByRole("button", { name: "다음" }));

  // 3단계: 기초 정보 미리 넣기
  expect(
    await screen.findByRole("heading", {
      level: 1,
      name: "기관의 기초 정보를 미리 넣어 둡니다",
    }),
  ).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "나중에 하기" }));

  // 이미 열린 세션으로 메인 화면 진입
  expect(await screen.findByTestId("main-screen")).toHaveTextContent(
    "메인 화면: 관리자1",
  );
});

test("StaffFlow는 PIN 복구 뒤 3단계 기초 정보 화면을 띄우지 않고 로그인으로 돌아간다", async () => {
  const user = userEvent.setup();
  vi.mocked(fetchSetupStatus).mockResolvedValue({ needed: false });
  vi.mocked(listLoginStaff).mockResolvedValue({
    items: [{ id: 1, name: "관리자1", role: "admin" }],
  });
  vi.mocked(recover).mockResolvedValue({
    recoveryCode: "NEW1-CODE-2345-6789",
  });

  renderWithQuery(<StaffFlow />);

  await user.click(
    await screen.findByRole("button", { name: "PIN을 잊었어요" }),
  );
  await user.selectOptions(await screen.findByLabelText("관리자 이름"), "1");
  await user.type(screen.getByLabelText("복구 코드"), "K7QF-M2XA-9TRE-HW4C");
  await user.type(screen.getByLabelText("새 PIN (숫자 6~8자리)"), "654321");
  await user.type(screen.getByLabelText("새 PIN 한 번 더"), "654321");
  await user.click(screen.getByRole("button", { name: "PIN 다시 정하기" }));

  expect(await screen.findByText("NEW1-CODE-2345-6789")).toBeInTheDocument();
  expect(screen.queryByText("기초 정보 미리 넣기")).not.toBeInTheDocument();

  await user.click(
    screen.getByRole("checkbox", { name: "인쇄하거나 적어 두었습니다" }),
  );
  await user.click(screen.getByRole("button", { name: "다음" }));

  expect(
    await screen.findByRole("button", { name: "로그인" }),
  ).toBeInTheDocument();
});
