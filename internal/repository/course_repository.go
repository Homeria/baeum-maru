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
	InstructorName string
	ClassroomName  string
	Capacity       int
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
		courseID, err := createCourse(ctx, tx, strings.TrimSpace(params.CourseTitle), categoryID)
		if err != nil {
			return err
		}
		instructorID, err := ensureOptionalNamed(ctx, tx, "instructors", strings.TrimSpace(params.InstructorName))
		if err != nil {
			return err
		}
		classroomID, err := ensureOptionalNamed(ctx, tx, "classrooms", strings.TrimSpace(params.ClassroomName))
		if err != nil {
			return err
		}

		result, err := tx.ExecContext(ctx, `
INSERT INTO course_offerings (term_id, course_id, instructor_id, classroom_id, capacity, status, note)
VALUES (?, ?, ?, ?, ?, 'open', NULLIF(?, ''));
`, termID, courseID, instructorID, classroomID, params.Capacity, strings.TrimSpace(params.Note))
		if err != nil {
			return fmt.Errorf("insert course offering: %w", err)
		}

		offeringID, err = result.LastInsertId()
		if err != nil {
			return fmt.Errorf("read course offering id: %w", err)
		}
		if _, err := tx.ExecContext(ctx, `
INSERT INTO course_meetings (offering_id, weekday, start_time, end_time)
VALUES (?, ?, ?, ?);
`, offeringID, params.Weekday, strings.TrimSpace(params.StartTime), strings.TrimSpace(params.EndTime)); err != nil {
			return fmt.Errorf("insert course meeting: %w", err)
		}
		return nil
	})
	if err != nil {
		return domain.CourseOffering{}, err
	}
	return r.GetOffering(ctx, offeringID)
}

func (r *CourseRepository) GetOffering(ctx context.Context, id int64) (domain.CourseOffering, error) {
	row := r.db.QueryRowContext(ctx, offeringSelectSQL()+`
WHERE co.id = ?
GROUP BY co.id, cm.id
ORDER BY cm.weekday, cm.start_time
LIMIT 1;
`, id)
	return scanCourseOffering(row)
}

func (r *CourseRepository) ListOfferings(ctx context.Context, limit int) ([]domain.CourseOffering, error) {
	if limit <= 0 || limit > 200 {
		limit = 100
	}

	rows, err := r.db.QueryContext(ctx, offeringSelectSQL()+`
GROUP BY co.id, cm.id
ORDER BY t.name, c.title, co.id
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
		offerings = append(offerings, offering)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate course offerings: %w", err)
	}
	return offerings, nil
}

func ensureTerm(ctx context.Context, tx *sql.Tx, name string) (int64, error) {
	if name == "" {
		name = "기본 회차"
	}
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO terms (name, status) VALUES (?, 'open');", name); err != nil {
		return 0, fmt.Errorf("ensure term: %w", err)
	}
	return selectID(ctx, tx, "terms", "name", name)
}

func ensureCategory(ctx context.Context, tx *sql.Tx, name string) (int64, error) {
	if name == "" {
		name = "미분류"
	}
	if _, err := tx.ExecContext(ctx, "INSERT OR IGNORE INTO course_categories (name) VALUES (?);", name); err != nil {
		return 0, fmt.Errorf("ensure category: %w", err)
	}
	return selectID(ctx, tx, "course_categories", "name", name)
}

func createCourse(ctx context.Context, tx *sql.Tx, title string, categoryID int64) (int64, error) {
	result, err := tx.ExecContext(ctx, "INSERT INTO courses (title, category_id) VALUES (?, ?);", title, categoryID)
	if err != nil {
		return 0, fmt.Errorf("insert course: %w", err)
	}
	id, err := result.LastInsertId()
	if err != nil {
		return 0, fmt.Errorf("read course id: %w", err)
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

func selectID(ctx context.Context, tx *sql.Tx, table string, column string, value string) (int64, error) {
	var id int64
	if err := tx.QueryRowContext(ctx, fmt.Sprintf("SELECT id FROM %s WHERE %s = ?;", table, column), value).Scan(&id); err != nil {
		return 0, fmt.Errorf("select %s id: %w", table, err)
	}
	return id, nil
}

func offeringSelectSQL() string {
	return `
SELECT co.id,
       t.id,
       t.name,
       t.status,
       t.max_registrations_per_member,
       c.id,
       c.title,
       cc.name,
       COALESCE(i.name, ''),
       COALESCE(cl.name, ''),
       co.capacity,
       co.registration_enabled,
       co.status,
       cm.weekday,
       cm.start_time,
       cm.end_time,
       COUNT(r.id)
FROM course_offerings co
JOIN terms t ON t.id = co.term_id
JOIN courses c ON c.id = co.course_id
LEFT JOIN course_categories cc ON cc.id = c.category_id
LEFT JOIN instructors i ON i.id = co.instructor_id
LEFT JOIN classrooms cl ON cl.id = co.classroom_id
LEFT JOIN course_meetings cm ON cm.offering_id = co.id
LEFT JOIN registrations r ON r.offering_id = co.id AND r.status != 'cancelled'
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
		&offering.InstructorName,
		&offering.ClassroomName,
		&offering.Capacity,
		&registrationEnabled,
		&offering.Status,
		&offering.Weekday,
		&offering.StartTime,
		&offering.EndTime,
		&offering.RegistrationCount,
	); err != nil {
		return domain.CourseOffering{}, fmt.Errorf("scan course offering: %w", err)
	}
	offering.RegistrationEnabled = registrationEnabled == 1
	return offering, nil
}
