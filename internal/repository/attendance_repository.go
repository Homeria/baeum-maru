package repository

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"strings"

	"github.com/Homeria/baeum-maru/internal/database"
	"github.com/Homeria/baeum-maru/internal/domain"
)

type AttendanceRepository struct {
	db *sql.DB
}

type CreateAttendanceSessionParams struct {
	OfferingID  int64
	SessionDate string
	Note        string
}

type SaveAttendanceRecordParams struct {
	SessionID      int64
	RegistrationID int64
	Status         string
	Note           string
}

func NewAttendanceRepository(db *sql.DB) *AttendanceRepository {
	return &AttendanceRepository{db: db}
}

func (r *AttendanceRepository) CreateSession(ctx context.Context, params CreateAttendanceSessionParams) (domain.AttendanceSession, error) {
	result, err := r.db.ExecContext(ctx, `
INSERT INTO attendance_sessions (offering_id, session_date, note)
VALUES (?, ?, NULLIF(?, ''));
`, params.OfferingID, strings.TrimSpace(params.SessionDate), strings.TrimSpace(params.Note))
	if err != nil {
		return domain.AttendanceSession{}, fmt.Errorf("insert attendance session: %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return domain.AttendanceSession{}, fmt.Errorf("read attendance session id: %w", err)
	}
	return r.GetSession(ctx, id)
}

func (r *AttendanceRepository) GetSession(ctx context.Context, id int64) (domain.AttendanceSession, error) {
	row := r.db.QueryRowContext(ctx, attendanceSessionSelectSQL()+`
WHERE s.id = ?;
`, id)
	return scanAttendanceSession(row)
}

func (r *AttendanceRepository) ListSessions(ctx context.Context, offeringID int64, limit int) ([]domain.AttendanceSession, error) {
	if limit <= 0 {
		limit = 50
	}
	if limit > 500 {
		limit = 500
	}

	rows, err := r.db.QueryContext(ctx, attendanceSessionSelectSQL()+`
WHERE ? = 0 OR s.offering_id = ?
ORDER BY s.session_date DESC, s.id DESC
LIMIT ?;
`, offeringID, offeringID, limit)
	if err != nil {
		return nil, fmt.Errorf("list attendance sessions: %w", err)
	}
	defer rows.Close()

	var sessions []domain.AttendanceSession
	for rows.Next() {
		session, err := scanAttendanceSession(rows)
		if err != nil {
			return nil, err
		}
		sessions = append(sessions, session)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate attendance sessions: %w", err)
	}
	return sessions, nil
}

func (r *AttendanceRepository) ListConfirmedByOffering(ctx context.Context, offeringID int64) ([]domain.Registration, error) {
	rows, err := r.db.QueryContext(ctx, registrationSelectSQL()+`
WHERE r.offering_id = ?
  AND r.status = 'confirmed'
ORDER BY m.name, r.id;
`, offeringID)
	if err != nil {
		return nil, fmt.Errorf("list confirmed registrations: %w", err)
	}
	defer rows.Close()

	return scanRegistrations(rows)
}

func (r *AttendanceRepository) ListRecordsBySession(ctx context.Context, sessionID int64) ([]domain.AttendanceRecord, error) {
	rows, err := r.db.QueryContext(ctx, `
SELECT COALESCE(ar.id, 0),
       s.id,
       s.offering_id,
       s.session_date,
       r.id,
       m.id,
       COALESCE(m.member_no, ''),
       m.name,
       COALESCE(ar.status, ''),
       COALESCE(ar.note, '')
FROM attendance_sessions s
JOIN registrations r ON r.offering_id = s.offering_id AND r.status = 'confirmed'
JOIN members m ON m.id = r.member_id
LEFT JOIN attendance_records ar ON ar.attendance_session_id = s.id AND ar.registration_id = r.id
WHERE s.id = ?
ORDER BY m.name, r.id;
`, sessionID)
	if err != nil {
		return nil, fmt.Errorf("list attendance records: %w", err)
	}
	defer rows.Close()

	var records []domain.AttendanceRecord
	for rows.Next() {
		var record domain.AttendanceRecord
		if err := rows.Scan(
			&record.ID,
			&record.SessionID,
			&record.OfferingID,
			&record.SessionDate,
			&record.RegistrationID,
			&record.MemberID,
			&record.MemberNo,
			&record.MemberName,
			&record.Status,
			&record.Note,
		); err != nil {
			return nil, fmt.Errorf("scan attendance record: %w", err)
		}
		records = append(records, record)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate attendance records: %w", err)
	}
	return records, nil
}

func (r *AttendanceRepository) SaveRecord(ctx context.Context, params SaveAttendanceRecordParams) (domain.AttendanceRecord, error) {
	var saved domain.AttendanceRecord
	err := database.WithTx(ctx, r.db, func(tx *sql.Tx) error {
		sessionOfferingID, err := readSessionOfferingID(ctx, tx, params.SessionID)
		if err != nil {
			return err
		}
		if err := ensureConfirmedRegistrationForOffering(ctx, tx, params.RegistrationID, sessionOfferingID); err != nil {
			return err
		}

		if _, err := tx.ExecContext(ctx, `
INSERT INTO attendance_records (attendance_session_id, registration_id, status, note)
VALUES (?, ?, ?, NULLIF(?, ''))
ON CONFLICT(attendance_session_id, registration_id)
DO UPDATE SET status = excluded.status,
              note = excluded.note;
`, params.SessionID, params.RegistrationID, params.Status, strings.TrimSpace(params.Note)); err != nil {
			return fmt.Errorf("upsert attendance record: %w", err)
		}

		row := tx.QueryRowContext(ctx, attendanceRecordSelectSQL()+`
WHERE ar.attendance_session_id = ? AND ar.registration_id = ?;
`, params.SessionID, params.RegistrationID)
		record, err := scanSavedAttendanceRecord(row)
		if err != nil {
			return err
		}
		saved = record
		return nil
	})
	if err != nil {
		return domain.AttendanceRecord{}, err
	}
	return saved, nil
}

func attendanceSessionSelectSQL() string {
	return `
SELECT s.id,
       s.offering_id,
       t.name,
       c.title,
       s.session_date,
       COALESCE(s.note, '')
FROM attendance_sessions s
JOIN course_offerings co ON co.id = s.offering_id
JOIN courses c ON c.id = co.course_id
JOIN terms t ON t.id = co.term_id
`
}

func attendanceRecordSelectSQL() string {
	return `
SELECT ar.id,
       ar.attendance_session_id,
       s.offering_id,
       s.session_date,
       ar.registration_id,
       m.id,
       COALESCE(m.member_no, ''),
       m.name,
       ar.status,
       COALESCE(ar.note, '')
FROM attendance_records ar
JOIN attendance_sessions s ON s.id = ar.attendance_session_id
JOIN registrations r ON r.id = ar.registration_id
JOIN members m ON m.id = r.member_id
`
}

func scanAttendanceSession(scanner interface{ Scan(dest ...any) error }) (domain.AttendanceSession, error) {
	var session domain.AttendanceSession
	if err := scanner.Scan(
		&session.ID,
		&session.OfferingID,
		&session.TermName,
		&session.CourseTitle,
		&session.SessionDate,
		&session.Note,
	); err != nil {
		return domain.AttendanceSession{}, fmt.Errorf("scan attendance session: %w", err)
	}
	return session, nil
}

func scanSavedAttendanceRecord(scanner interface{ Scan(dest ...any) error }) (domain.AttendanceRecord, error) {
	var record domain.AttendanceRecord
	if err := scanner.Scan(
		&record.ID,
		&record.SessionID,
		&record.OfferingID,
		&record.SessionDate,
		&record.RegistrationID,
		&record.MemberID,
		&record.MemberNo,
		&record.MemberName,
		&record.Status,
		&record.Note,
	); err != nil {
		return domain.AttendanceRecord{}, fmt.Errorf("scan saved attendance record: %w", err)
	}
	return record, nil
}

func readSessionOfferingID(ctx context.Context, tx *sql.Tx, sessionID int64) (int64, error) {
	var offeringID int64
	if err := tx.QueryRowContext(ctx, "SELECT offering_id FROM attendance_sessions WHERE id = ?;", sessionID).Scan(&offeringID); err != nil {
		return 0, fmt.Errorf("read attendance session offering: %w", err)
	}
	return offeringID, nil
}

func ensureConfirmedRegistrationForOffering(ctx context.Context, tx *sql.Tx, registrationID int64, offeringID int64) error {
	var status string
	var registrationOfferingID int64
	if err := tx.QueryRowContext(ctx, "SELECT status, offering_id FROM registrations WHERE id = ?;", registrationID).Scan(&status, &registrationOfferingID); err != nil {
		return fmt.Errorf("read attendance registration: %w", err)
	}
	if registrationOfferingID != offeringID {
		return errors.New("registration does not belong to attendance session offering")
	}
	if status != "confirmed" {
		return errors.New("attendance can be recorded only for confirmed registrations")
	}
	return nil
}
