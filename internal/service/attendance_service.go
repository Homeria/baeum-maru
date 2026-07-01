package service

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type AttendanceRepository interface {
	CreateSession(context.Context, repository.CreateAttendanceSessionParams) (domain.AttendanceSession, error)
	ListSessions(context.Context, int64, int) ([]domain.AttendanceSession, error)
	ListConfirmedByOffering(context.Context, int64) ([]domain.Registration, error)
	ListRecordsBySession(context.Context, int64) ([]domain.AttendanceRecord, error)
	SaveRecord(context.Context, repository.SaveAttendanceRecordParams) (domain.AttendanceRecord, error)
}

type AttendanceService struct {
	attendance AttendanceRepository
}

type AttendanceSessionInput struct {
	OfferingID  int64
	SessionDate string
	Note        string
}

type AttendanceRecordInput struct {
	SessionID      int64
	RegistrationID int64
	Status         string
	Note           string
}

func NewAttendanceService(attendance AttendanceRepository) *AttendanceService {
	return &AttendanceService{attendance: attendance}
}

func (s *AttendanceService) CreateSession(ctx context.Context, input AttendanceSessionInput) (domain.AttendanceSession, error) {
	if input.OfferingID <= 0 {
		return domain.AttendanceSession{}, errors.New("course offering id is required")
	}
	if _, err := time.Parse("2006-01-02", strings.TrimSpace(input.SessionDate)); err != nil {
		return domain.AttendanceSession{}, errors.New("attendance session date must be YYYY-MM-DD")
	}
	return s.attendance.CreateSession(ctx, repository.CreateAttendanceSessionParams{
		OfferingID:  input.OfferingID,
		SessionDate: strings.TrimSpace(input.SessionDate),
		Note:        strings.TrimSpace(input.Note),
	})
}

func (s *AttendanceService) ListSessions(ctx context.Context, offeringID int64, limit int) ([]domain.AttendanceSession, error) {
	return s.attendance.ListSessions(ctx, offeringID, limit)
}

func (s *AttendanceService) ListConfirmedByOffering(ctx context.Context, offeringID int64) ([]domain.Registration, error) {
	if offeringID <= 0 {
		return nil, nil
	}
	return s.attendance.ListConfirmedByOffering(ctx, offeringID)
}

func (s *AttendanceService) ListRecordsBySession(ctx context.Context, sessionID int64) ([]domain.AttendanceRecord, error) {
	if sessionID <= 0 {
		return nil, nil
	}
	return s.attendance.ListRecordsBySession(ctx, sessionID)
}

func (s *AttendanceService) SaveRecord(ctx context.Context, input AttendanceRecordInput) (domain.AttendanceRecord, error) {
	if input.SessionID <= 0 {
		return domain.AttendanceRecord{}, errors.New("attendance session id is required")
	}
	if input.RegistrationID <= 0 {
		return domain.AttendanceRecord{}, errors.New("registration id is required")
	}
	if !validAttendanceStatus(input.Status) {
		return domain.AttendanceRecord{}, errors.New("attendance status is invalid")
	}
	return s.attendance.SaveRecord(ctx, repository.SaveAttendanceRecordParams{
		SessionID:      input.SessionID,
		RegistrationID: input.RegistrationID,
		Status:         input.Status,
		Note:           strings.TrimSpace(input.Note),
	})
}

func validAttendanceStatus(status string) bool {
	switch status {
	case "present", "absent", "late", "excused":
		return true
	default:
		return false
	}
}
