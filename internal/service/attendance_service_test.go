package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

func TestAttendanceServiceCreatesSessionAndSavesRecord(t *testing.T) {
	repo := &fakeAttendanceRepository{}
	service := NewAttendanceService(repo)

	session, err := service.CreateSession(context.Background(), AttendanceSessionInput{
		OfferingID:  1,
		SessionDate: "2026-07-01",
		Note:        "1회차",
	})
	if err != nil {
		t.Fatalf("CreateSession() error = %v", err)
	}
	if session.SessionDate != "2026-07-01" {
		t.Fatalf("SessionDate = %q, want 2026-07-01", session.SessionDate)
	}

	record, err := service.SaveRecord(context.Background(), AttendanceRecordInput{
		SessionID:      2,
		RegistrationID: 3,
		Status:         "present",
	})
	if err != nil {
		t.Fatalf("SaveRecord() error = %v", err)
	}
	if record.Status != "present" {
		t.Fatalf("Status = %q, want present", record.Status)
	}
}

func TestAttendanceServiceRejectsInvalidStatus(t *testing.T) {
	service := NewAttendanceService(&fakeAttendanceRepository{})

	if _, err := service.SaveRecord(context.Background(), AttendanceRecordInput{SessionID: 1, RegistrationID: 2, Status: "bad"}); err == nil {
		t.Fatal("SaveRecord() error = nil, want invalid status error")
	}
}

type fakeAttendanceRepository struct {
	sessionParams repository.CreateAttendanceSessionParams
	recordParams  repository.SaveAttendanceRecordParams
}

func (f *fakeAttendanceRepository) CreateSession(_ context.Context, params repository.CreateAttendanceSessionParams) (domain.AttendanceSession, error) {
	f.sessionParams = params
	return domain.AttendanceSession{ID: 1, OfferingID: params.OfferingID, SessionDate: params.SessionDate, Note: params.Note}, nil
}

func (f *fakeAttendanceRepository) ListSessions(context.Context, int64, int) ([]domain.AttendanceSession, error) {
	return []domain.AttendanceSession{{ID: 1}}, nil
}

func (f *fakeAttendanceRepository) ListConfirmedByOffering(context.Context, int64) ([]domain.Registration, error) {
	return []domain.Registration{{ID: 3, Status: "confirmed"}}, nil
}

func (f *fakeAttendanceRepository) ListRecordsBySession(context.Context, int64) ([]domain.AttendanceRecord, error) {
	return []domain.AttendanceRecord{{ID: 4, Status: "present"}}, nil
}

func (f *fakeAttendanceRepository) SaveRecord(_ context.Context, params repository.SaveAttendanceRecordParams) (domain.AttendanceRecord, error) {
	f.recordParams = params
	return domain.AttendanceRecord{ID: 4, SessionID: params.SessionID, RegistrationID: params.RegistrationID, Status: params.Status}, nil
}
