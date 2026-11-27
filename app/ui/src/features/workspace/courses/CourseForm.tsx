import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createCourse, deleteCourse, updateCourse } from "./api";
import type { CourseResponse } from "../../../api/generated/CourseResponse";
import type { RoomResponse } from "../../../api/generated/RoomResponse";
import type { SubjectResponse } from "../../../api/generated/SubjectResponse";
import { DeleteDialog } from "../../../components/dialog/DeleteDialog";
import styles from "../../../components/form/form.module.css";
import { NO_FORM_ERRORS } from "../../../components/form/formErrors";
import {
  EMPTY_COURSE_FORM,
  WEEKDAYS,
  addScheduleRow,
  courseFormErrorsFrom,
  courseFormValuesFrom,
  courseName,
  firstErrorField,
  roomLabel,
  timeRangeLabel,
  toCourseRequest,
  type CourseField,
  type CourseFormErrors,
  type CourseFormValues,
  type ScheduleRowValues,
  type TimeRange,
} from "./courseFormData";
import rowStyles from "./CourseForm.module.css";

/** {@link CourseForm}의 속성. */
export interface CourseFormProps {
  /** 새 강좌를 넣을 학기. */
  termId: number;
  /** 고를 수 있는 과목. */
  subjects: readonly SubjectResponse[];
  /** 고를 수 있는 강의실. */
  rooms: readonly RoomResponse[];
  /** 강의실 이름에 건물을 붙일지. */
  showBuilding: boolean;
  /** 이 학기 강좌들이 이미 쓰는 시간대. 줄마다 골라 시각을 채운다. 비었으면 고르는 칸을 숨긴다. */
  usedTimes: readonly TimeRange[];
  /** 고칠 강좌. 없으면 새 강좌 등록. */
  editing?: CourseResponse;
  /** 처음에 보여 줄 알림. 예: 강좌를 지운 뒤 등록 칸으로 돌아왔을 때 */
  initialNotice?: string | null;
  /** 고치기를 끝내고 등록 칸으로 돌아간다. `notice`는 돌아간 칸에 보여 줄 알림. */
  onClose: (notice?: string) => void;
}

/**
 * 강좌 입력 칸. 새 강좌 등록과 강좌 고치기를 함께 맡는다.
 *
 * 수업 시간은 줄마다 요일, 시작·종료 시각, 강의실(선택)을 넣는다. 줄을 더하면 앞 줄의 시각과
 * 강의실을 그대로 쓴다.
 */
export function CourseForm({
  termId,
  subjects,
  rooms,
  showBuilding,
  usedTimes,
  editing,
  initialNotice,
  onClose,
}: CourseFormProps) {
  const queryClient = useQueryClient();
  const [values, setValues] = useState<CourseFormValues>(() =>
    editing ? courseFormValuesFrom(editing) : EMPTY_COURSE_FORM,
  );
  const [errors, setErrors] = useState<CourseFormErrors>(NO_FORM_ERRORS);
  const [notice, setNotice] = useState<string | null>(initialNotice ?? null);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  // 수업 시간 칸은 첫 줄의 요일에 커서를 둔다
  const inputs = useRef<Partial<Record<CourseField, HTMLElement | null>>>({});

  const showErrors = (error: unknown) => {
    const next = courseFormErrorsFrom(error);
    setErrors(next);
    const field = firstErrorField(next.fields);
    if (field) {
      inputs.current[field]?.focus();
    }
  };

  /** 저장한 강좌의 이름. 등록은 응답에 이름이 없어 고른 과목에서 만든다. */
  const nameOf = (values: CourseFormValues) =>
    courseName({
      subjectName:
        subjects.find((subject) => String(subject.id) === values.subjectId)
          ?.name ?? "",
      level: values.level.trim(),
      section: values.section.trim(),
    });

  const save = useMutation({
    mutationFn: (values: CourseFormValues) => {
      const request = toCourseRequest(values);
      return editing
        ? updateCourse(editing.id, request).then(courseName)
        : createCourse(termId, request).then(() => nameOf(values));
    },
    onSuccess: (name) => {
      void queryClient.invalidateQueries({ queryKey: ["courses", termId] });
      setErrors(NO_FORM_ERRORS);
      if (editing) {
        setNotice(`저장했습니다: ${name}`);
      } else {
        setValues(EMPTY_COURSE_FORM);
        setNotice(`등록했습니다: ${name}`);
        inputs.current.subjectId?.focus();
      }
    },
    onError: showErrors,
  });

  const remove = useMutation({
    mutationFn: deleteCourse,
    onSuccess: () => {
      setConfirmingDelete(false);
      onClose(`지웠습니다: ${editing ? courseName(editing) : ""}`);
      void queryClient.invalidateQueries({ queryKey: ["courses", termId] });
    },
    onError: (error) => {
      setConfirmingDelete(false);
      showErrors(error);
    },
  });

  const busy = save.isPending || remove.isPending;

  /** 칸 하나를 바꾸고 그 칸의 오류를 지운다. */
  const change = (patch: Partial<CourseFormValues>, field: CourseField) => {
    setValues((current) => ({ ...current, ...patch }));
    setErrors((current) => ({
      ...current,
      fields: { ...current.fields, [field]: undefined },
    }));
    setNotice(null);
  };

  const changeRow = (key: number, patch: Partial<ScheduleRowValues>) =>
    change(
      {
        schedules: values.schedules.map((row) =>
          row.key === key ? { ...row, ...patch } : row,
        ),
      },
      "schedules",
    );

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
    name: Exclude<CourseField, "schedules">,
    label: string,
    required: boolean,
    control: (props: {
      id: string;
      "aria-invalid": boolean;
      "aria-describedby": string | undefined;
    }) => ReactNode,
  ) => {
    const id = `course-${name}`;
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

  const textInput = (
    name: "level" | "section",
    label: string,
    placeholder: string,
  ) =>
    field(name, label, false, (props) => (
      <input
        {...props}
        ref={(element) => {
          inputs.current[name] = element;
        }}
        className={styles.input}
        type="text"
        autoComplete="off"
        placeholder={placeholder}
        value={values[name]}
        onChange={(event) => change({ [name]: event.target.value }, name)}
      />
    ));

  const scheduleError = errors.fields.schedules;

  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <h2 className={styles.title}>
        {editing ? "강좌 고치기" : "새 강좌 등록"}
      </h2>

      {field("subjectId", "과목", true, (props) => (
        <>
          <select
            {...props}
            ref={(element) => {
              inputs.current.subjectId = element;
            }}
            className={styles.input}
            value={values.subjectId}
            onChange={(event) =>
              change({ subjectId: event.target.value }, "subjectId")
            }
          >
            <option value="">고르세요</option>
            {subjects.map((subject) => (
              <option key={subject.id} value={subject.id}>
                {subject.name}
              </option>
            ))}
          </select>
          {subjects.length === 0 && (
            <p className={rowStyles.empty}>
              과목이 없습니다. 기초 정보 탭에서 과목을 먼저 등록해 주세요
            </p>
          )}
        </>
      ))}
      {textInput("level", "수준", "예: 초급, 중급")}
      {textInput("section", "반", "예: 1반, 오전반")}
      <p className={rowStyles.hint}>
        같은 과목을 여러 강좌로 열 때 수준이나 반으로 나눕니다. 하나만 열면 비워
        두세요.
      </p>
      {field("capacity", "정원", true, (props) => (
        // 숫자 칸이라 글자는 들어가지 않음. 단위는 칸 밖에 붙여 "명"을 쓰지 않아도 되게
        <div className={rowStyles.withUnit}>
          <input
            {...props}
            ref={(element) => {
              inputs.current.capacity = element;
            }}
            className={`${styles.input} ${rowStyles.number}`}
            type="number"
            inputMode="numeric"
            min={1}
            max={200}
            placeholder="예: 20"
            value={values.capacity}
            onChange={(event) =>
              change({ capacity: event.target.value }, "capacity")
            }
          />
          명
        </div>
      ))}

      <fieldset
        className={styles.field}
        aria-describedby={scheduleError ? "course-schedules-error" : undefined}
      >
        <legend className={styles.label}>
          수업 시간<span className={styles.required}>필수</span>
        </legend>
        <div className={rowStyles.rows}>
          {values.schedules.map((row, index) => {
            const number = index + 1;
            return (
              <div key={row.key} className={rowStyles.row}>
                <span className={rowStyles.rowNumber}>{number}</span>
                <select
                  ref={(element) => {
                    if (index === 0) {
                      inputs.current.schedules = element;
                    }
                  }}
                  className={styles.input}
                  aria-label={`${number}번째 요일`}
                  value={row.weekday}
                  onChange={(event) =>
                    changeRow(row.key, { weekday: event.target.value })
                  }
                >
                  {WEEKDAYS.map((label, day) => (
                    <option key={label} value={day + 1}>
                      {label}
                    </option>
                  ))}
                </select>
                {usedTimes.length > 0 && (
                  <select
                    className={styles.input}
                    aria-label={`${number}번째 시간대 고르기`}
                    // 지금 시각이 목록의 시간대와 같으면 그것을, 아니면 안내 문구를 보여 줌
                    value={
                      usedTimes.some(
                        (time) => timeRangeLabel(time) === timeRangeLabel(row),
                      )
                        ? timeRangeLabel(row)
                        : ""
                    }
                    onChange={(event) => {
                      const range = usedTimes.find(
                        (time) => timeRangeLabel(time) === event.target.value,
                      );
                      if (range) {
                        changeRow(row.key, {
                          start: range.start,
                          end: range.end,
                        });
                      }
                    }}
                  >
                    <option value="" hidden>
                      쓴 시간대
                    </option>
                    {usedTimes.map((time) => (
                      <option
                        key={timeRangeLabel(time)}
                        value={timeRangeLabel(time)}
                      >
                        {timeRangeLabel(time)}
                      </option>
                    ))}
                  </select>
                )}
                <input
                  className={styles.input}
                  type="time"
                  aria-label={`${number}번째 시작 시각`}
                  value={row.start}
                  onChange={(event) =>
                    changeRow(row.key, { start: event.target.value })
                  }
                />
                ~
                <input
                  className={styles.input}
                  type="time"
                  aria-label={`${number}번째 종료 시각`}
                  value={row.end}
                  onChange={(event) =>
                    changeRow(row.key, { end: event.target.value })
                  }
                />
                <select
                  className={`${styles.input} ${rowStyles.room}`}
                  aria-label={`${number}번째 강의실`}
                  value={row.roomId}
                  onChange={(event) =>
                    changeRow(row.key, { roomId: event.target.value })
                  }
                >
                  <option value="">강의실 없음</option>
                  {rooms.map((room) => (
                    <option key={room.id} value={room.id}>
                      {roomLabel(room, showBuilding)}
                    </option>
                  ))}
                </select>
                <button
                  type="button"
                  className={styles.secondary}
                  aria-label={`${number}번째 수업 시간 빼기`}
                  // 수업 시간은 한 줄 이상이라 마지막 줄은 뺄 수 없음 (BR-T4)
                  disabled={values.schedules.length === 1}
                  onClick={() =>
                    change(
                      {
                        schedules: values.schedules.filter(
                          (other) => other.key !== row.key,
                        ),
                      },
                      "schedules",
                    )
                  }
                >
                  빼기
                </button>
              </div>
            );
          })}
        </div>
        <button
          type="button"
          className={styles.secondary}
          onClick={() =>
            change({ schedules: addScheduleRow(values.schedules) }, "schedules")
          }
        >
          수업 시간 더하기
        </button>
        {scheduleError && (
          <ul id="course-schedules-error" className={rowStyles.errors}>
            {scheduleError.split("\n").map((message) => (
              <li key={message}>{message}</li>
            ))}
          </ul>
        )}
      </fieldset>

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
              새 강좌 등록으로
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
          title={`${courseName(editing)} 강좌를 지울까요?`}
          deleting={remove.isPending}
          onConfirm={() => remove.mutate(editing.id)}
          onCancel={() => setConfirmingDelete(false)}
        />
      )}
    </form>
  );
}
