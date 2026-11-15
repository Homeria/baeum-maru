import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import type { RoomResponse } from "../../api/generated/RoomResponse";
import { createRoom } from "../../api/rooms";
import styles from "../../components/form/form.module.css";
import { FLOORS, floorLabel } from "./floor";
import {
  buildingNames,
  firstErrorField,
  lastUsedBuilding,
  roomFormErrorsFrom,
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
}

/**
 * 새 강의실 등록 칸.
 *
 * 등록하면 이름만 비운다. 같은 건물, 같은 층의 강의실을 이어서 넣기 쉽게 건물과 층은 남긴다.
 */
export function RoomForm({ rooms }: RoomFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<RoomFormValues>(() => ({
    name: "",
    building: lastUsedBuilding(rooms),
    floor: "",
  }));
  const [errors, setErrors] = useState<RoomFormErrors>(NO_ERRORS);
  const [notice, setNotice] = useState<string | null>(null);
  const inputs = useRef<
    Partial<Record<RoomField, HTMLInputElement | HTMLSelectElement | null>>
  >({});

  const save = useMutation({
    mutationFn: (values: RoomFormValues) => {
      const request = toRoomRequest(values);
      return createRoom(request).then(() => request.name.trim());
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["rooms"] });
      setErrors(NO_ERRORS);
      setValues((current) => ({ ...current, name: "" }));
      setNotice(`등록했습니다: ${name}`);
      inputs.current.name?.focus();
    },
    onError: (error) => {
      const next = roomFormErrorsFrom(error);
      setErrors(next);
      const field = firstErrorField(next.fields);
      if (field) {
        inputs.current[field]?.focus();
      }
    },
  });

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
    if (save.isPending) {
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
      <h2 className={styles.title}>새 강의실 등록</h2>

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
        <button
          type="submit"
          className={styles.submit}
          disabled={save.isPending}
        >
          {save.isPending ? "등록하는 중…" : "등록"}
        </button>
      </div>
    </form>
  );
}
