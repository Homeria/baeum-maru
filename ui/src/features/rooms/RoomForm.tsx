import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { RoomResponse } from "../../api/generated/RoomResponse";
import { createRoom, deleteRoom, updateRoom } from "../../api/rooms";
import { DeleteDialog } from "../../components/dialog/DeleteDialog";
import styles from "../../components/form/form.module.css";
import { FLOORS, floorLabel } from "./floor";
import {
  buildingNames,
  firstErrorField,
  lastUsedBuilding,
  roomFormErrorsFrom,
  roomFormValuesFrom,
  toRoomRequest,
  type RoomField,
  type RoomFormErrors,
  type RoomFormValues,
} from "./roomFormData";

const NO_ERRORS: RoomFormErrors = { fields: {}, form: null };

const BUILDING_LIST_ID = "room-building-names";

/** {@link RoomForm}의 속성. */
export interface RoomFormProps {
  /** 지금 있는 강의실. 건물 칸의 처음 값과 자동완성에 쓴다. */
  rooms: readonly RoomResponse[];
  /** 고칠 강의실. 없으면 새 강의실 등록. */
  editing?: RoomResponse;
  /** 처음에 보여 줄 알림. 예: 강의실을 지운 뒤 등록 칸으로 돌아왔을 때 */
  initialNotice?: string | null;
  /** 고치기를 끝내고 등록 칸으로 돌아간다. `notice`는 돌아간 칸에 보여 줄 알림. */
  onClose: (notice?: string) => void;
}

/**
 * 강의실 입력 칸. 새 강의실 등록과 강의실 고치기를 함께 맡는다.
 *
 * 등록하면 이름만 비운다. 같은 건물, 같은 층의 강의실을 이어서 넣기 쉽게 건물과 층은 남긴다.
 */
export function RoomForm({
  rooms,
  editing,
  initialNotice,
  onClose,
}: RoomFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<RoomFormValues>(() =>
    editing
      ? roomFormValuesFrom(editing)
      : { name: "", building: lastUsedBuilding(rooms), floor: "" },
  );
  const [errors, setErrors] = useState<RoomFormErrors>(NO_ERRORS);
  const [notice, setNotice] = useState<string | null>(initialNotice ?? null);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const inputs = useRef<
    Partial<Record<RoomField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const showErrors = (error: unknown) => {
    const next = roomFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  const save = useMutation({
    mutationFn: (values: RoomFormValues) => {
      const request = toRoomRequest(values);
      return editing
        ? updateRoom(editing.id, request).then((room) => room.name)
        : createRoom(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["rooms"] });
      setErrors(NO_ERRORS);
      if (editing) {
        setNotice(`저장했습니다: ${name}`);
      } else {
        setValues((current) => ({ ...current, name: "" }));
        setNotice(`등록했습니다: ${name}`);
        inputs.current.name?.focus();
      }
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteRoom,
    onSuccess: () => {
      setConfirmingDelete(false);
      onClose(`지웠습니다: ${editing?.name ?? ""}`);
      void queryClient.invalidateQueries({ queryKey: ["rooms"] });
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  const change = (field: RoomField, value: string) => {
    setValues((current) => ({ ...current, [field]: value }));
    // 고치기 시작한 칸의 오류는 지움
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
    setNotice(null);
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (busy) {
      return;
    }
    setNotice(null);
    save.mutate(values);
  };

  /** 칸 이름표와 오류 문구 사이에 입력 부품을 끼운다. */
  const field = (
    name: RoomField,
    label: string,
    required: boolean,
    control: (props: {
      id: string;
      "aria-invalid": boolean;
      "aria-describedby": string | undefined;
    }) => ReactNode,
  ) => {
    const id = `room-${name}`;
    const errorId = `${id}-error`;
    const error = errors.fields[name];
    return (
      <div className={styles.field}>
        <label htmlFor={id} className={styles.label}>
          {label}
          {required && <span className={styles.required}>필수</span>}
        </label>
        {control({
          id,
          "aria-invalid": error !== undefined,
          "aria-describedby": error ? errorId : undefined,
        })}
        {error && (
          <p id={errorId} className={styles.fieldError}>
            {error}
          </p>
        )}
      </div>
    );
  };

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>
        {editing ? "강의실 고치기" : "새 강의실 등록"}
      </h2>

      {field("name", "이름", true, (props) => (
        <input
          {...props}
          ref={(element) => {
            inputs.current.name = element;
          }}
          className={styles.input}
          type="text"
          autoComplete="off"
          value={values.name}
          onChange={(event) => change("name", event.target.value)}
        />
      ))}

      {field("building", "건물", true, (props) => (
        <>
          <input
            {...props}
            ref={(element) => {
              inputs.current.building = element;
            }}
            className={styles.input}
            type="text"
            autoComplete="off"
            list={BUILDING_LIST_ID}
            value={values.building}
            onChange={(event) => change("building", event.target.value)}
          />
          <datalist id={BUILDING_LIST_ID}>
            {buildingNames(rooms).map((building) => (
              <option key={building} value={building} />
            ))}
          </datalist>
        </>
      ))}

      {field("floor", "층", false, (props) => (
        <select
          {...props}
          ref={(element) => {
            inputs.current.floor = element;
          }}
          className={styles.input}
          value={values.floor}
          onChange={(event) => change("floor", event.target.value)}
        >
          <option value="">(층 없음)</option>
          {FLOORS.map((floor) => (
            <option key={floor} value={String(floor)}>
              {floorLabel(floor)}
            </option>
          ))}
        </select>
      ))}

      {errors.form && (
        <p className={styles.formError} role="alert">
          {errors.form}
        </p>
      )}
      {notice && (
        <p className={styles.notice} role="status">
          {notice}
        </p>
      )}

      <div className={styles.actions}>
        <button type="submit" className={styles.submit} disabled={busy}>
          {editing
            ? save.isPending
              ? "저장하는 중…"
              : "저장"
            : save.isPending
              ? "등록하는 중…"
              : "등록"}
        </button>
        {editing && (
          <>
            <button
              type="button"
              className={styles.secondary}
              onClick={() => onClose()}
              disabled={busy}
            >
              새 강의실 등록으로
            </button>
            <button
              type="button"
              className={styles.danger}
              onClick={() => setConfirmingDelete(true)}
              disabled={busy}
            >
              지우기
            </button>
          </>
        )}
      </div>

      {editing && confirmingDelete && (
        <DeleteDialog
          title={`${editing.name} 강의실을 지울까요?`}
          deleting={remove.isPending}
          onConfirm={() => remove.mutate(editing.id)}
          onCancel={() => setConfirmingDelete(false)}
        />
      )}
    </form>
  );
}
