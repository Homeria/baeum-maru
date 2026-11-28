import { useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import { Card } from "../../../components/surface/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/surface/DataTable";
import { NoticeBanner } from "../../../components/surface/NoticeBanner";
import { Button } from "../../../components/ui/Button";
import { SelectField } from "../../../components/ui/SelectField";
import { TextField } from "../../../components/ui/TextField";
import { createRoom, deleteRoom } from "../../workspace/rooms/api";
import { FLOORS, floorLabel } from "../../workspace/rooms/floor";
import {
  DEFAULT_BUILDING,
  buildingNames,
  firstErrorField,
  roomFormErrorsFrom,
  toRoomRequest,
  type RoomField,
  type RoomFormErrors,
} from "../../workspace/rooms/roomFormData";
import styles from "./SetupBasicsPage.module.css";

const ROOM_BUILDING_LIST_ID = "setup-basics-buildings";

/** {@link RoomsStep}의 속성. */
export interface RoomsStepProps {
  rooms: readonly RoomResponse[];
  loading: boolean;
  error: string | null;
  onRetry: () => void;
}

/** 3단계 하위 화면 1: 건물·층 마지막 값을 유지하며 강의실을 연속 입력한다. */
export function RoomsStep({ rooms, loading, error, onRetry }: RoomsStepProps) {
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
    const field = firstErrorField(next.fields);
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
