import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { PriorityReasonResponse } from "../../../api/generated/PriorityReasonResponse";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import {
  SplitAuthLayout,
  type SplitAuthStep,
} from "../../../components/shell/SplitAuthLayout";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { Checkbox } from "../../../components/ui/Checkbox";
import { SelectField } from "../../../components/ui/SelectField";
import { StatusPill } from "../../../components/ui/StatusPill";
import { TextField } from "../../../components/ui/TextField";
import {
  createPriorityReason,
  deletePriorityReason,
  listPriorityReasons,
} from "../../workspace/priority-reasons/api";
import {
  priorityReasonFormErrorsFrom,
  type PriorityReasonFormErrors,
} from "../../workspace/priority-reasons/priorityReasonFormData";
import { createRoom, deleteRoom, listRooms } from "../../workspace/rooms/api";
import { FLOORS, floorLabel } from "../../workspace/rooms/floor";
import {
  DEFAULT_BUILDING,
  buildingNames,
  firstErrorField as firstRoomErrorField,
  roomFormErrorsFrom,
  toRoomRequest,
  type RoomField,
  type RoomFormErrors,
} from "../../workspace/rooms/roomFormData";
import {
  createSubject,
  deleteSubject,
  listSubjects,
} from "../../workspace/subjects/api";
import {
  categoryNames,
  firstErrorField as firstSubjectErrorField,
  subjectFormErrorsFrom,
  toSubjectRequest,
  type SubjectField,
  type SubjectFormErrors,
} from "../../workspace/subjects/subjectFormData";
import {
  PRESET_PRIORITY_REASONS,
  customPriorityReasons,
  findPresetReason,
  type BasicsStep,
} from "./setupBasicsData";
import styles from "./SetupBasicsPage.module.css";

const ROOM_BUILDING_LIST_ID = "setup-basics-buildings";
const SUBJECT_CATEGORY_LIST_ID = "setup-basics-categories";

/** {@link SetupBasicsPage}의 속성. */
export interface SetupBasicsPageProps {
  /** 기초 정보 미리 넣기를 끝내거나 나중에 하기를 눌렀다. */
  onDone: () => void;
}

/**
 * 호스트 첫 실행 3단계: 기초 정보 미리 넣기(강의실 → 과목·우선 배정 사유).
 *
 * 기존 `/api/rooms`, `/api/subjects`, `/api/priority-reasons` API로 즉시 저장·삭제한다.
 * 선택 단계라서 어느 화면에서든 "나중에 하기"로 메인 화면에 바로 들어갈 수 있다.
 */
export function SetupBasicsPage({ onDone }: SetupBasicsPageProps) {
  const [step, setStep] = useState<BasicsStep>("rooms");
  const rooms = useQuery({ queryKey: ["rooms"], queryFn: listRooms });
  const subjects = useQuery({ queryKey: ["subjects"], queryFn: listSubjects });
  const reasons = useQuery({
    queryKey: ["priority-reasons"],
    queryFn: listPriorityReasons,
  });

  const roomItems = rooms.data?.items ?? [];
  const subjectItems = subjects.data?.items ?? [];
  const reasonItems = reasons.data?.items ?? [];

  const steps: readonly SplitAuthStep[] = [
    {
      label: "관리자 계정 만들기",
      description: "관리자 계정을 만들었습니다.",
      status: "done",
    },
    {
      label: "복구 코드 보관",
      description: "복구 코드를 확인했습니다.",
      status: "done",
    },
    {
      label: "기초 정보 미리 넣기",
      description:
        step === "rooms"
          ? "강의실(건물·층) → 과목·우선 배정 사유 순으로 넣습니다."
          : `강의실 ${roomItems.length}개 등록 · 과목과 우선 배정 사유를 넣습니다.`,
      status: "current",
    },
  ];

  const tip =
    step === "rooms" ? (
      <div className={styles.panelTip}>
        <p className={styles.panelTipTitle}>지금 다 넣지 않아도 됩니다</p>
        <p className={styles.panelTipText}>
          건너뛰거나 일부만 넣고, 나중에 [기초 정보] 탭에서 언제든 더하고 고칠
          수 있습니다.
        </p>
      </div>
    ) : (
      <div className={styles.panelTip}>
        <p className={styles.panelTipTitle}>강좌는 시작한 뒤 엽니다</p>
        <p className={styles.panelTipText}>
          학기마다 바뀌는 정원·수업 시간은 시작 후 [강좌] 탭에서 편하게
          등록합니다.
        </p>
      </div>
    );

  return (
    <SplitAuthLayout
      panelBadge="선택 단계"
      panelTitle={
        step === "rooms"
          ? "기관의 기초 정보를 미리 넣어 둡니다"
          : "과목과 우선 배정 사유를 정합니다"
      }
      panelDescription={
        step === "rooms"
          ? "강의실과 과목처럼 학기가 바뀌어도 그대로 쓰는 정보를 먼저 등록하면 바로 강좌를 열 수 있습니다."
          : "자주 여는 과목과 우선 배정 사유(직전 학기 반장·보훈·장애 등)를 넣어 두면 첫 학기 강좌를 바로 만들 수 있습니다."
      }
      steps={steps}
      panelFooter={tip}
      frameless
    >
      <div className={styles.subStepBar}>
        <nav aria-label="기초 정보 단계" className={styles.subStepNav}>
          <button
            type="button"
            aria-current={step === "rooms" ? "step" : undefined}
            className={[
              styles.subStepPill,
              step === "rooms"
                ? styles.subStepPillActive
                : styles.subStepPillDone,
            ].join(" ")}
            onClick={() => setStep("rooms")}
          >
            1. 강의실 · {roomItems.length}개
          </button>
          <span className={styles.subStepArrow} aria-hidden="true">
            →
          </span>
          <button
            type="button"
            aria-current={step === "subjects" ? "step" : undefined}
            className={[
              styles.subStepPill,
              step === "subjects" ? styles.subStepPillActive : "",
            ]
              .filter(Boolean)
              .join(" ")}
            onClick={() => setStep("subjects")}
          >
            2. 과목 · 우선 배정 사유
          </button>
        </nav>
        <p className={styles.subStepHint}>
          {step === "rooms"
            ? "건물과 층은 마지막 값이 유지되어 연속 입력하기 쉽습니다"
            : "언제든 [기초 정보] 탭에서 바꿀 수 있습니다"}
        </p>
      </div>

      {step === "rooms" ? (
        <RoomsSetupCard
          rooms={roomItems}
          loading={rooms.isPending}
          error={rooms.error?.message ?? null}
          onRetry={() => void rooms.refetch()}
        />
      ) : (
        <div className={styles.subjectsGrid}>
          <SubjectsSetupCard
            subjects={subjectItems}
            loading={subjects.isPending}
            error={subjects.error?.message ?? null}
            onRetry={() => void subjects.refetch()}
          />
          <PriorityReasonsSetupCard
            reasons={reasonItems}
            loading={reasons.isPending}
            error={reasons.error?.message ?? null}
            onRetry={() => void reasons.refetch()}
          />
        </div>
      )}

      <Card>
        <div className={styles.footerBar}>
          {step === "rooms" ? (
            <>
              <Button variant="secondary" onClick={onDone}>
                나중에 하기
              </Button>
              <Button variant="primary" onClick={() => setStep("subjects")}>
                다음: 과목 · 우선 배정
              </Button>
            </>
          ) : (
            <>
              <Button variant="secondary" onClick={() => setStep("rooms")}>
                이전: 강의실
              </Button>
              <div className={styles.footerRight}>
                <Button variant="secondary" onClick={onDone}>
                  나중에 하기
                </Button>
                <Button variant="primary" onClick={onDone}>
                  설정 마치고 시작하기
                </Button>
              </div>
            </>
          )}
        </div>
      </Card>
    </SplitAuthLayout>
  );
}

interface RoomsSetupCardProps {
  rooms: readonly RoomResponse[];
  loading: boolean;
  error: string | null;
  onRetry: () => void;
}

function RoomsSetupCard({
  rooms,
  loading,
  error,
  onRetry,
}: RoomsSetupCardProps) {
  const queryClient = useQueryClient();
  const [building, setBuilding] = useState(DEFAULT_BUILDING);
  const [floor, setFloor] = useState("");
  const [name, setName] = useState("");
  const [errors, setErrors] = useState<RoomFormErrors>(NO_FORM_ERRORS);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const inputs = useRef<
    Partial<Record<RoomField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const showErrors = (failure: unknown) => {
    const next = roomFormErrorsFrom(failure);
    setErrors(next);
    const field = firstRoomErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const add = useMutation({
    mutationFn: () => createRoom(toRoomRequest({ building, floor, name })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["rooms"] });
      setErrors(NO_FORM_ERRORS);
      // 건물과 층은 유지하고 이름만 비워 연속 입력하기 쉽게 한다
      setName("");
      inputs.current.name?.focus();
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteRoom,
    onMutate: (id) => {
      setDeletingId(id);
      setErrors(NO_FORM_ERRORS);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["rooms"] });
    },
    onError: showErrors,
    onSettled: () => setDeletingId(null),
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (add.isPending) {
      return;
    }
    add.mutate();
  };

  const columns: readonly DataTableColumn<RoomResponse>[] = [
    {
      key: "building",
      header: "건물",
      width: "24%",
      render: (room) => room.building,
    },
    {
      key: "floor",
      header: "층",
      width: "18%",
      render: (room) => floorLabel(room.floor) || "—",
    },
    {
      key: "name",
      header: "강의실 이름",
      isRowHeader: true,
      render: (room) => room.name,
    },
    {
      key: "actions",
      header: "",
      align: "right",
      width: "96px",
      isAction: true,
      render: (room) => (
        <Button
          size="sm"
          variant="danger"
          aria-label={`${room.name} 빼기`}
          disabled={deletingId === room.id}
          onClick={() => remove.mutate(room.id)}
        >
          빼기
        </Button>
      ),
    },
  ];

  const buildings = buildingNames(rooms);

  return (
    <Card
      title="강의실"
      actions={
        <span className={styles.countBadge}>{rooms.length}개 등록됨</span>
      }
    >
      <div className={styles.cardStack}>
        <form className={styles.roomForm} onSubmit={submit} noValidate>
          <TextField
            id="setup-room-building"
            ref={(element) => {
              inputs.current.building = element;
            }}
            label="건물"
            list={ROOM_BUILDING_LIST_ID}
            value={building}
            error={errors.fields.building}
            onChange={(event) => {
              setBuilding(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, building: undefined },
              }));
            }}
          />
          <datalist id={ROOM_BUILDING_LIST_ID}>
            {buildings.map((item) => (
              <option key={item} value={item} />
            ))}
          </datalist>

          <SelectField
            id="setup-room-floor"
            ref={(element) => {
              inputs.current.floor = element;
            }}
            label="층"
            value={floor}
            error={errors.fields.floor}
            onChange={(event) => {
              setFloor(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, floor: undefined },
              }));
            }}
          >
            <option value="">선택 안 함</option>
            {FLOORS.map((item) => (
              <option key={item} value={String(item)}>
                {floorLabel(item)}
              </option>
            ))}
          </SelectField>

          <TextField
            id="setup-room-name"
            ref={(element) => {
              inputs.current.name = element;
            }}
            label="강의실 이름"
            required
            placeholder="예: 3강의실, 정보화실, 대강당"
            value={name}
            error={errors.fields.name}
            onChange={(event) => {
              setName(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, name: undefined },
              }));
            }}
          />

          <div className={styles.addButtonWrap}>
            <Button
              type="submit"
              variant="primary"
              busy={add.isPending}
              busyLabel="더하는 중…"
            >
              더하기
            </Button>
          </div>
        </form>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}

        <div className={styles.tableScroll}>
          <DataTable
            caption="등록한 강의실 목록"
            columns={columns}
            rows={rooms}
            getRowKey={(room) => room.id}
            loading={loading}
            loadingText="강의실을 불러오는 중…"
            error={error}
            onRetry={onRetry}
            emptyMessage="아직 등록한 강의실이 없습니다. 위에서 건물·층과 강의실 이름을 넣어 더해 주세요."
          />
        </div>
      </div>
    </Card>
  );
}

interface SubjectsSetupCardProps {
  subjects: readonly SubjectResponse[];
  loading: boolean;
  error: string | null;
  onRetry: () => void;
}

function SubjectsSetupCard({
  subjects,
  loading,
  error,
  onRetry,
}: SubjectsSetupCardProps) {
  const queryClient = useQueryClient();
  const [category, setCategory] = useState("");
  const [name, setName] = useState("");
  const [errors, setErrors] = useState<SubjectFormErrors>(NO_FORM_ERRORS);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const inputs = useRef<Partial<Record<SubjectField, HTMLInputElement | null>>>(
    {},
  );

  const showErrors = (failure: unknown) => {
    const next = subjectFormErrorsFrom(failure);
    setErrors(next);
    const field = firstSubjectErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const add = useMutation({
    mutationFn: () => createSubject(toSubjectRequest({ category, name })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
      setErrors(NO_FORM_ERRORS);
      // 분류는 유지하고 과목 이름만 비워 같은 분류의 과목을 연속 입력하기 쉽게 한다
      setName("");
      inputs.current.name?.focus();
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteSubject,
    onMutate: (id) => {
      setDeletingId(id);
      setErrors(NO_FORM_ERRORS);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["subjects"] });
    },
    onError: showErrors,
    onSettled: () => setDeletingId(null),
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (add.isPending) {
      return;
    }
    add.mutate();
  };

  const columns: readonly DataTableColumn<SubjectResponse>[] = [
    {
      key: "category",
      header: "분류",
      width: "34%",
      render: (subject) => subject.category ?? "—",
    },
    {
      key: "name",
      header: "과목 이름",
      isRowHeader: true,
      render: (subject) => subject.name,
    },
    {
      key: "actions",
      header: "",
      align: "right",
      width: "96px",
      isAction: true,
      render: (subject) => (
        <Button
          size="sm"
          variant="danger"
          aria-label={`${subject.name} 빼기`}
          disabled={deletingId === subject.id}
          onClick={() => remove.mutate(subject.id)}
        >
          빼기
        </Button>
      ),
    },
  ];

  const categories = categoryNames(subjects);

  return (
    <Card
      title="과목"
      actions={
        <span className={styles.countBadge}>{subjects.length}개 등록됨</span>
      }
    >
      <div className={styles.cardStack}>
        <form className={styles.subjectForm} onSubmit={submit} noValidate>
          <TextField
            id="setup-subject-category"
            ref={(element) => {
              inputs.current.category = element;
            }}
            label="분류"
            list={SUBJECT_CATEGORY_LIST_ID}
            placeholder="예: 서예·미술"
            value={category}
            error={errors.fields.category}
            onChange={(event) => {
              setCategory(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, category: undefined },
              }));
            }}
          />
          <datalist id={SUBJECT_CATEGORY_LIST_ID}>
            {categories.map((item) => (
              <option key={item} value={item} />
            ))}
          </datalist>

          <TextField
            id="setup-subject-name"
            ref={(element) => {
              inputs.current.name = element;
            }}
            label="과목 이름"
            required
            placeholder="예: 서예, 스마트폰"
            value={name}
            error={errors.fields.name}
            onChange={(event) => {
              setName(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, name: undefined },
              }));
            }}
          />

          <div className={styles.addButtonWrap}>
            <Button
              type="submit"
              variant="primary"
              busy={add.isPending}
              busyLabel="더하는 중…"
            >
              더하기
            </Button>
          </div>
        </form>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}

        <div className={styles.tableScroll}>
          <DataTable
            caption="등록한 과목 목록"
            columns={columns}
            rows={subjects}
            getRowKey={(subject) => subject.id}
            loading={loading}
            loadingText="과목을 불러오는 중…"
            error={error}
            onRetry={onRetry}
            emptyMessage="아직 등록한 과목이 없습니다. 위에서 과목 이름을 넣어 더해 주세요."
          />
        </div>
      </div>
    </Card>
  );
}

interface PriorityReasonsSetupCardProps {
  reasons: readonly PriorityReasonResponse[];
  loading: boolean;
  error: string | null;
  onRetry: () => void;
}

function PriorityReasonsSetupCard({
  reasons,
  loading,
  error,
  onRetry,
}: PriorityReasonsSetupCardProps) {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [errors, setErrors] =
    useState<PriorityReasonFormErrors>(NO_FORM_ERRORS);
  const [busyPreset, setBusyPreset] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const nameInput = useRef<HTMLInputElement | null>(null);

  const add = useMutation({
    mutationFn: (reasonName: string) =>
      createPriorityReason({ name: reasonName, method: "guaranteed" }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
      setErrors(NO_FORM_ERRORS);
      setName("");
      nameInput.current?.focus();
    },
    onError: (failure) => {
      setErrors(priorityReasonFormErrorsFrom(failure));
      nameInput.current?.focus();
    },
  });

  const toggle = useMutation({
    mutationFn: async ({
      presetName,
      existing,
      checked,
    }: {
      presetName: string;
      existing: PriorityReasonResponse | undefined;
      checked: boolean;
    }) => {
      if (checked && !existing) {
        await createPriorityReason({ name: presetName, method: "guaranteed" });
      } else if (!checked && existing) {
        await deletePriorityReason(existing.id);
      }
    },
    onMutate: ({ presetName }) => {
      setBusyPreset(presetName);
      setErrors(NO_FORM_ERRORS);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
    },
    onError: (failure) => setErrors(priorityReasonFormErrorsFrom(failure)),
    onSettled: () => setBusyPreset(null),
  });

  const remove = useMutation({
    mutationFn: deletePriorityReason,
    onMutate: (id) => {
      setDeletingId(id);
      setErrors(NO_FORM_ERRORS);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["priority-reasons"] });
    },
    onError: (failure) => setErrors(priorityReasonFormErrorsFrom(failure)),
    onSettled: () => setDeletingId(null),
  });

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (add.isPending) {
      return;
    }
    add.mutate(name);
  };

  const customReasons = customPriorityReasons(reasons);

  return (
    <Card
      title="우선 배정 사유"
      actions={
        <span className={styles.countBadge}>{reasons.length}개 선택됨</span>
      }
    >
      <div className={styles.cardStack}>
        <form className={styles.reasonForm} onSubmit={submit} noValidate>
          <TextField
            id="setup-priority-reason-name"
            ref={nameInput}
            label="사유 이름"
            placeholder="예: 다자녀"
            value={name}
            error={errors.fields.name}
            onChange={(event) => {
              setName(event.target.value);
              setErrors((current) => ({
                ...current,
                fields: { ...current.fields, name: undefined },
              }));
            }}
          />
          <div className={styles.addButtonWrap}>
            <Button
              type="submit"
              variant="primary"
              busy={add.isPending}
              busyLabel="더하는 중…"
            >
              더하기
            </Button>
          </div>
        </form>

        {errors.form && <NoticeBanner tone="error" message={errors.form} />}

        {error ? (
          <NoticeBanner
            tone="error"
            message={error}
            actionLabel="다시 해 보기"
            onAction={onRetry}
          />
        ) : loading ? (
          <NoticeBanner message="우선 배정 사유를 불러오는 중…" />
        ) : (
          <>
            <ul className={styles.presetList} aria-label="추천 우선 배정 사유">
              {PRESET_PRIORITY_REASONS.map((preset) => {
                const existing = findPresetReason(reasons, preset);
                const checked = existing !== undefined;
                return (
                  <li
                    key={preset}
                    className={[
                      styles.presetRow,
                      checked ? styles.presetRowChecked : "",
                    ]
                      .filter(Boolean)
                      .join(" ")}
                  >
                    <Checkbox
                      label={preset}
                      checked={checked}
                      disabled={busyPreset === preset}
                      onChange={(event) =>
                        toggle.mutate({
                          presetName: preset,
                          existing,
                          checked: event.target.checked,
                        })
                      }
                    />
                    <StatusPill tone={checked ? "primary" : "neutral"}>
                      자리 보장
                    </StatusPill>
                  </li>
                );
              })}
            </ul>

            {customReasons.length > 0 && (
              <ul
                className={styles.customReasonList}
                aria-label="직접 추가한 우선 배정 사유"
              >
                {customReasons.map((reason) => (
                  <li key={reason.id} className={styles.customReasonRow}>
                    <div className={styles.customReasonMain}>
                      <span className={styles.customReasonName}>
                        {reason.name}
                      </span>
                      <StatusPill tone="primary">자리 보장</StatusPill>
                    </div>
                    <Button
                      size="sm"
                      variant="danger"
                      aria-label={`${reason.name} 빼기`}
                      disabled={deletingId === reason.id}
                      onClick={() => remove.mutate(reason.id)}
                    >
                      빼기
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </>
        )}

        <p className={styles.reasonHint}>
          추첨 없이 자리를 보장할 대상을 고르거나 위에서 직접 추가하세요.
        </p>
      </div>
    </Card>
  );
}
