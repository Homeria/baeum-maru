package repository

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/domain"
)

type CourseRepository struct {
	db *sql.DB
}

type CreateCourseOfferingParams struct {
	TermName       string
	CategoryName   string
	CourseTitle    string
	DisplayName    string
	LevelLabel     string
	SectionLabel   string
	InstructorName string
	ClassroomName  string
	Capacity       int
	CapacityType   string
	MaleCapacity   int
	FemaleCapacity int
	Weekday        int
	StartTime      string
	EndTime        string
	Note           string
}

type UpdateCourseOfferingParams struct {
	ID             int64
	TermName       string
	CategoryName   string
	CourseTitle    string
	DisplayName    string
	LevelLabel     string
	SectionLabel   string
	InstructorName string
	ClassroomName  string
	Capacity       int
	CapacityType   string
	MaleCapacity   int
	FemaleCapacity int
	Weekday        int
	StartTime      string
	EndTime        string
	Note           string
}

func NewCourseRepository(db *sql.DB) *CourseRepository {
	return &CourseRepository{db: db}
}

func (r *CourseRepository) CreateOffering(ctx context.Context, params CreateCourseOfferingParams) (domain.CourseOffering, error) {
	var offeringID int64
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		termID, err := ensureTerm(ctx, tx, strings.TrimSpace(params.TermName))
		if err != nil {
			return err
		}
		categoryID, err := ensureCategory(ctx, tx, strings.TrimSpace(params.CategoryName))
		if err != nil {
			return err
		}
		courseID, err := ensureCourse(ctx, tx, strings.TrimSpace(params.CourseTitle), categoryID)
		if err != nil {
			return err
		}
		instructorID, err := ensureOptionalNamed(ctx, tx, "instructors", strings.TrimSpace(params.InstructorName))
		if err != nil {
			return err
		}
		locationID, err := ensureCourseLocation(ctx, tx, strings.TrimSpace(params.ClassroomName))
		if err != nil {
			return err
		}
		capacityType, capacityTotal, maleCapacity, femaleCapacity := normalizeCapacityParams(params.CapacityType, params.Capacity, params.MaleCapacity, params.FemaleCapacity)

		result, err := tx.ExecContext(ctx, `
INSERT INTO course_offerings (
  term_id, course_id, display_name, level_label, section_label, instructor_id, location_id,
  capacity_type, capacity_total, male_capacity, female_capacity, status, note
)
VALUES (?, ?, ?, NULLIF(?, ''), NULLIF(?, ''), ?, ?, ?, ?, ?, ?, 'open', NULLIF(?, ''));
`, termID, courseID, displayName(params.DisplayName, params.CourseTitle), strings.TrimSpace(params.LevelLabel), strings.TrimSpace(params.SectionLabel), instructorID, locationID, capacityType, capacityTotal, maleCapacity, femaleCapacity, strings.TrimSpace(params.Note))
		if err != nil {
			return fmt.Errorf("insert course offering: %w", err)
		}

		offeringID, err = result.LastInsertId()
		if err != nil {
			return fmt.Errorf("read course offering id: %w", err)
		}
		return replaceOfferingSchedule(ctx, tx, offeringID, params.Weekday, params.StartTime, params.EndTime)
	})
	if err != nil {
		return domain.CourseOffering{}, err
	}
	return r.GetOffering(ctx, offeringID)
}

func (r *CourseRepository) UpdateOffering(ctx context.Context, params UpdateCourseOfferingParams) (domain.CourseOffering, error) {
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		var activeRegistrations int
		if err := tx.QueryRowContext(ctx, `
SELECT COUNT(*)
FROM registrations
WHERE offering_id = ? AND status != 'cancelled';
`, params.ID).Scan(&activeRegistrations); err != nil {
			return fmt.Errorf("count active registrations: %w", err)
		}
		capacityType, capacityTotal, maleCapacity, femaleCapacity := normalizeCapacityParams(params.CapacityType, params.Capacity, params.MaleCapacity, params.FemaleCapacity)
		if capacityType != "open" && capacityTotal.Valid && int(capacityTotal.Int64) < activeRegistrations {
			return fmt.Errorf("course capacity cannot be less than active registration count %d", activeRegistrations)
		}

		termID, err := ensureTerm(ctx, tx, strings.TrimSpace(params.TermName))
		if err != nil {
			return err
		}
		categoryID, err := ensureCategory(ctx, tx, strings.TrimSpace(params.CategoryName))
		if err != nil {
			return err
		}
		instructorID, err := ensureOptionalNamed(ctx, tx, "instructors", strings.TrimSpace(params.InstructorName))
		if err != nil {
			return err
		}
		locationID, err := ensureCourseLocation(ctx, tx, strings.TrimSpace(params.ClassroomName))
		if err != nil {
			return err
		}

		var courseID int64
		if err := tx.QueryRowContext(ctx, `
SELECT course_id
FROM course_offerings
WHERE id = ?;
`, params.ID).Scan(&courseID); err != nil {
			return fmt.Errorf("select course offering %d: %w", params.ID, err)
		}
		if _, err := tx.ExecContext(ctx, `
UPDATE courses
SET name = ?,
    category_id = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, strings.TrimSpace(params.CourseTitle), categoryID, courseID); err != nil {
			return fmt.Errorf("update course: %w", err)
		}

		result, err := tx.ExecContext(ctx, `
UPDATE course_offerings
SET term_id = ?,
    display_name = ?,
    level_label = NULLIF(?, ''),
    section_label = NULLIF(?, ''),
    instructor_id = ?,
    location_id = ?,
    capacity_type = ?,
    capacity_total = ?,
    male_capacity = ?,
    female_capacity = ?,
    note = NULLIF(?, ''),
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;
`, termID, displayName(params.DisplayName, params.CourseTitle), strings.TrimSpace(params.LevelLabel), strings.TrimSpace(params.SectionLabel), instructorID, locationID, capacityType, capacityTotal, maleCapacity, femaleCapacity, strings.TrimSpace(params.Note), params.ID)
		if err != nil {
			return fmt.Errorf("update course offering %d: %w", params.ID, err)
		}
		affected, err := result.RowsAffected()
		if err != nil {
			return fmt.Errorf("read affected course offering rows: %w", err)
		}
		if affected == 0 {
			return sql.ErrNoRows
		}
		return replaceOfferingSchedule(ctx, tx, params.ID, params.Weekday, params.StartTime, params.EndTime)
	})
	if err != nil {
		return domain.CourseOffering{}, err
	}
	return r.GetOffering(ctx, params.ID)
}

func (r *CourseRepository) GetOffering(ctx context.Context, id int64) (domain.CourseOffering, error) {
	row := r.db.QueryRowContext(ctx, offeringSelectSQL()+`
WHERE co.id = ?;
`, id)
	offering, err := scanCourseOffering(row)
	if err != nil {
		return domain.CourseOffering{}, err
	}
	schedules, err := r.listSchedules(ctx, offering.ID)
	if err != nil {
		return domain.CourseOffering{}, err
	}
	offering.Schedules = schedules
	if len(schedules) > 0 {
		offering.Weekday = schedules[0].Weekday
		offering.StartTime = schedules[0].StartTime
		offering.EndTime = schedules[0].EndTime
		offering.ScheduleText = scheduleLabel(offering)
	}
	return offering, nil
}

func (r *CourseRepository) ListOfferings(ctx context.Context, limit int) ([]domain.CourseOffering, error) {
	if limit <= 0 {
		limit = 100
	}
	if limit > 10000 {
		limit = 10000
	}

	rows, err := r.db.QueryContext(ctx, offeringSelectSQL()+`
ORDER BY t.name, c.name, co.display_name, co.id
LIMIT ?;
`, limit)
	if err != nil {
		return nil, fmt.Errorf("list course offerings: %w", err)
	}
	defer rows.Close()

	var offerings []domain.CourseOffering
	for rows.Next() {
		offering, err := scanCourseOffering(rows)
		if err != nil {
			return nil, err
		}
		schedules, err := r.listSchedules(ctx, offering.ID)
		if err != nil {
			return nil, err
		}
		offering.Schedules = schedules
		offering.ScheduleText = scheduleLabel(offering)
		offerings = append(offerings, offering)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate course offerings: %w", err)
	}
	return offerings, nil
}

func (r *CourseRepository) listSchedules(ctx context.Context, offeringID int64) ([]domain.CourseSchedule, error) {
	rows, err := r.db.QueryContext(ctx, `
SELECT cs.id,
       cs.weekday,
       ts.id,
       ts.name,
       ts.start_time,
       ts.end_time
FROM course_schedules cs
JOIN time_slots ts ON ts.id = cs.time_slot_id
WHERE cs.offering_id = ?
ORDER BY cs.weekday, ts.start_time, ts.end_time, cs.id;
`, offeringID)
	if err != nil {
		return nil, fmt.Errorf("list course schedules: %w", err)
	}
	defer rows.Close()

	var schedules []domain.CourseSchedule
	for rows.Next() {
		var schedule domain.CourseSchedule
		if err := rows.Scan(
			&schedule.ID,
			&schedule.Weekday,
			&schedule.TimeSlotID,
			&schedule.TimeSlot,
			&schedule.StartTime,
			&schedule.EndTime,
		); err != nil {
			return nil, fmt.Errorf("scan course schedule: %w", err)
		}
		schedule.Weekday = weekdayFromDB(schedule.Weekday)
		schedules = append(schedules, schedule)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate course schedules: %w", err)
	}
	return schedules, nil
}

func ensureTerm(ctx context.Context, tx *sql.Tx, name string) (int64, error) {
	if name == "" {
		name = "Default Term"
	}
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO terms (name, status) VALUES (?, 'open');", name); err != nil {
		return 0, fmt.Errorf("ensure term: %w", err)
	}
	return selectID(ctx, tx, "terms", "name", name)
}

func ensureCategory(ctx context.Context, tx *sql.Tx, name string) (int64, error) {
	if name == "" {
		name = "Uncategorized"
	}
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO course_categories (name) VALUES (?);", name); err != nil {
		return 0, fmt.Errorf("ensure category: %w", err)
	}
	return selectID(ctx, tx, "course_categories", "name", name)
}

func ensureCourse(ctx context.Context, tx *sql.Tx, name string, categoryID int64) (int64, error) {
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO courses (name, category_id) VALUES (?, ?);", name, categoryID); err != nil {
		return 0, fmt.Errorf("ensure course: %w", err)
	}
	var id int64
	if err := tx.QueryRowContext(ctx, "SELECT id FROM courses WHERE name = ? AND category_id = ?;", name, categoryID).Scan(&id); err != nil {
		return 0, fmt.Errorf("select course id: %w", err)
	}
	return id, nil
}

func ensureOptionalNamed(ctx context.Context, tx *sql.Tx, table string, name string) (sql.NullInt64, error) {
	if name == "" {
		return sql.NullInt64{}, nil
	}
	if _, err := tx.ExecContext(ctx, fmt.Sprintf("INSERT OR IGNORE INTO %s (name) VALUES (?);", table), name); err != nil {
		return sql.NullInt64{}, fmt.Errorf("ensure %s: %w", table, err)
	}
	id, err := selectID(ctx, tx, table, "name", name)
	if err != nil {
		return sql.NullInt64{}, err
	}
	return sql.NullInt64{Int64: id, Valid: true}, nil
}

func ensureCourseLocation(ctx context.Context, tx *sql.Tx, name string) (sql.NullInt64, error) {
	if name == "" {
		return sql.NullInt64{}, nil
	}
	if _, err := tx.ExecContext(ctx, `
INSERT OR IGNORE INTO locations (name, is_active)
VALUES (?, 1);
`, name); err != nil {
		return sql.NullInt64{}, fmt.Errorf("ensure location: %w", err)
	}
	id, err := selectID(ctx, tx, "locations", "name", name)
	if err != nil {
		return sql.NullInt64{}, err
	}
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO location_roles (name) VALUES (?);", domain.LocationTypeClassroom); err != nil {
		return sql.NullInt64{}, fmt.Errorf("ensure classroom role: %w", err)
	}
	roleID, err := selectID(ctx, tx, "location_roles", "name", domain.LocationTypeClassroom)
	if err != nil {
		return sql.NullInt64{}, err
	}
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO location_role_assignments (location_id, role_id) VALUES (?, ?);", id, roleID); err != nil {
		return sql.NullInt64{}, fmt.Errorf("assign classroom role: %w", err)
	}
	return sql.NullInt64{Int64: id, Valid: true}, nil
}

func replaceOfferingSchedule(ctx context.Context, tx *sql.Tx, offeringID int64, weekday int, startTime string, endTime string) error {
	if _, err := tx.ExecContext(ctx, "DELETE FROM course_schedules WHERE offering_id = ?;", offeringID); err != nil {
		return fmt.Errorf("delete course schedules: %w", err)
	}
	slotID, err := ensureTimeSlot(ctx, tx, strings.TrimSpace(startTime), strings.TrimSpace(endTime))
	if err != nil {
		return err
	}
	if _, err := tx.ExecContext(ctx, `
INSERT INTO course_schedules (offering_id, weekday, time_slot_id)
VALUES (?, ?, ?);
`, offeringID, weekdayToDB(weekday), slotID); err != nil {
		return fmt.Errorf("insert course schedule: %w", err)
	}
	return nil
}

func ensureTimeSlot(ctx context.Context, tx *sql.Tx, startTime string, endTime string) (int64, error) {
	name := startTime + "-" + endTime
	if _, err := tx.ExecContext(ctx, `
INSERT OR IGNORE INTO time_slots (name, start_time, end_time)
VALUES (?, ?, ?);
`, name, startTime, endTime); err != nil {
		return 0, fmt.Errorf("ensure time slot: %w", err)
	}
	return selectID(ctx, tx, "time_slots", "name", name)
}

func selectID(ctx context.Context, tx *sql.Tx, table string, column string, value string) (int64, error) {
	var id int64
	if err := tx.QueryRowContext(ctx, fmt.Sprintf("SELECT id FROM %s WHERE %s = ?;", table, column), value).Scan(&id); err != nil {
		return 0, fmt.Errorf("select %s id: %w", table, err)
	}
	return id, nil
}

func normalizeCapacityParams(capacityType string, capacity int, maleCapacity int, femaleCapacity int) (string, sql.NullInt64, sql.NullInt64, sql.NullInt64) {
	capacityType = strings.TrimSpace(capacityType)
	if capacityType == "" {
		capacityType = "fixed"
	}
	switch capacityType {
	case "open":
		return capacityType, sql.NullInt64{}, sql.NullInt64{}, sql.NullInt64{}
	case "gender_split":
		total := maleCapacity + femaleCapacity
		return capacityType, sql.NullInt64{Int64: int64(total), Valid: true}, sql.NullInt64{Int64: int64(maleCapacity), Valid: true}, sql.NullInt64{Int64: int64(femaleCapacity), Valid: true}
	default:
		return "fixed", sql.NullInt64{Int64: int64(capacity), Valid: true}, sql.NullInt64{}, sql.NullInt64{}
	}
}

func displayName(displayName string, courseTitle string) string {
	displayName = strings.TrimSpace(displayName)
	if displayName != "" {
		return displayName
	}
	return strings.TrimSpace(courseTitle)
}

func weekdayToDB(weekday int) int {
	if weekday == 0 {
		return 7
	}
	return weekday
}

func weekdayFromDB(weekday int) int {
	if weekday == 7 {
		return 0
	}
	return weekday
}

func offeringSelectSQL() string {
	return `
SELECT co.id,
       t.id,
       t.name,
       t.status,
       t.max_registrations_per_member,
       c.id,
       c.name,
       cc.name,
       co.display_name,
       COALESCE(co.level_label, ''),
       COALESCE(co.section_label, ''),
       COALESCE(i.name, ''),
       COALESCE(l.id, 0),
       COALESCE(l.name, ''),
       COALESCE(l.floor_label, ''),
       co.capacity_type,
       COALESCE(co.capacity_total, 0),
       COALESCE(co.male_capacity, 0),
       COALESCE(co.female_capacity, 0),
       co.registration_enabled,
       co.status,
       COALESCE((
         SELECT cs.weekday
         FROM course_schedules cs
         JOIN time_slots ts ON ts.id = cs.time_slot_id
         WHERE cs.offering_id = co.id
         ORDER BY cs.weekday, ts.start_time
         LIMIT 1
       ), 1),
       COALESCE((
         SELECT ts.start_time
         FROM course_schedules cs
         JOIN time_slots ts ON ts.id = cs.time_slot_id
         WHERE cs.offering_id = co.id
         ORDER BY cs.weekday, ts.start_time
         LIMIT 1
       ), ''),
       COALESCE((
         SELECT ts.end_time
         FROM course_schedules cs
         JOIN time_slots ts ON ts.id = cs.time_slot_id
         WHERE cs.offering_id = co.id
         ORDER BY cs.weekday, ts.start_time
         LIMIT 1
       ), ''),
       COALESCE(co.note, ''),
       (
         SELECT COUNT(*)
         FROM registrations r
         WHERE r.offering_id = co.id AND r.status != 'cancelled'
       )
FROM course_offerings co
JOIN terms t ON t.id = co.term_id
JOIN courses c ON c.id = co.course_id
LEFT JOIN course_categories cc ON cc.id = c.category_id
LEFT JOIN instructors i ON i.id = co.instructor_id
LEFT JOIN locations l ON l.id = co.location_id
`
}

func scanCourseOffering(scanner interface{ Scan(dest ...any) error }) (domain.CourseOffering, error) {
	var offering domain.CourseOffering
	var registrationEnabled int
	if err := scanner.Scan(
		&offering.ID,
		&offering.TermID,
		&offering.TermName,
		&offering.TermStatus,
		&offering.MaxRegistrationsPerMember,
		&offering.CourseID,
		&offering.CourseTitle,
		&offering.CategoryName,
		&offering.DisplayName,
		&offering.LevelLabel,
		&offering.SectionLabel,
		&offering.InstructorName,
		&offering.LocationID,
		&offering.LocationName,
		&offering.FloorLabel,
		&offering.CapacityType,
		&offering.CapacityTotal,
		&offering.MaleCapacity,
		&offering.FemaleCapacity,
		&registrationEnabled,
		&offering.Status,
		&offering.Weekday,
		&offering.StartTime,
		&offering.EndTime,
		&offering.Note,
		&offering.RegistrationCount,
	); err != nil {
		return domain.CourseOffering{}, fmt.Errorf("scan course offering: %w", err)
	}
	offering.RegistrationEnabled = registrationEnabled == 1
	offering.Weekday = weekdayFromDB(offering.Weekday)
	offering.ClassroomName = offering.LocationName
	offering.Capacity = offering.CapacityTotal
	offering.CapacityLabel = capacityLabel(offering)
	offering.ScheduleText = scheduleLabel(offering)
	return offering, nil
}

func capacityLabel(offering domain.CourseOffering) string {
	switch offering.CapacityType {
	case "open":
		return "open"
	case "gender_split":
		return fmt.Sprintf("M%d/F%d", offering.MaleCapacity, offering.FemaleCapacity)
	default:
		return fmt.Sprintf("%d", offering.CapacityTotal)
	}
}

func scheduleLabel(offering domain.CourseOffering) string {
	if len(offering.Schedules) > 0 {
		parts := make([]string, 0, len(offering.Schedules))
		for _, schedule := range offering.Schedules {
			parts = append(parts, fmt.Sprintf("%d %s-%s", schedule.Weekday, schedule.StartTime, schedule.EndTime))
		}
		return strings.Join(parts, ", ")
	}
	if offering.StartTime == "" && offering.EndTime == "" {
		return ""
	}
	return fmt.Sprintf("%d %s-%s", offering.Weekday, offering.StartTime, offering.EndTime)
}
