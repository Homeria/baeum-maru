package service

import (
	"context"
	"errors"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type fakeRegistrationRepository struct {
	created     repository.CreateRegistrationParams
	cancelledID int64
	confirmedID int64
	activeItems []domain.RegistrationRuleItem
}

func (f *fakeRegistrationRepository) Create(_ context.Context, params repository.CreateRegistrationParams) (domain.Registration, error) {
	f.created = params
	return domain.Registration{ID: 1, MemberID: params.MemberID, OfferingID: params.OfferingID, Status: "applied"}, nil
}

func (f *fakeRegistrationRepository) Cancel(_ context.Context, id int64) (domain.Registration, error) {
	f.cancelledID = id
	return domain.Registration{ID: id, Status: "cancelled"}, nil
}

func (f *fakeRegistrationRepository) Confirm(_ context.Context, id int64) (domain.Registration, error) {
	f.confirmedID = id
	return domain.Registration{ID: id, Status: "confirmed"}, nil
}

func (f *fakeRegistrationRepository) CancelAndPromote(_ context.Context, id int64) (domain.RegistrationStatusChange, error) {
	f.cancelledID = id
	return domain.RegistrationStatusChange{
		Registration: domain.Registration{ID: id, Status: "cancelled"},
	}, nil
}

func (f *fakeRegistrationRepository) ListByMember(_ context.Context, memberID int64) ([]domain.Registration, error) {
	return []domain.Registration{{ID: 1, MemberID: memberID}}, nil
}

func (f *fakeRegistrationRepository) ListByOffering(_ context.Context, offeringID int64) ([]domain.Registration, error) {
	return []domain.Registration{{ID: 1, OfferingID: offeringID}}, nil
}

func (f *fakeRegistrationRepository) ListRecent(_ context.Context, _ int) ([]domain.Registration, error) {
	return []domain.Registration{{ID: 1}}, nil
}

func (f *fakeRegistrationRepository) ListActiveRuleItemsByMember(_ context.Context, _ int64) ([]domain.RegistrationRuleItem, error) {
	return f.activeItems, nil
}

type fakeMemberLookup struct{}

func (fakeMemberLookup) Get(_ context.Context, id int64) (domain.Member, error) {
	return domain.Member{ID: id, Name: "김배움"}, nil
}

type fakeCourseOfferingLookup struct {
	offering domain.CourseOffering
}

func (f fakeCourseOfferingLookup) GetOffering(_ context.Context, id int64) (domain.CourseOffering, error) {
	if f.offering.TermStatus == "" && f.offering.Status == "" && f.offering.CourseTitle == "" {
		return domain.CourseOffering{
			ID:                  id,
			TermID:              1,
			TermStatus:          "open",
			CourseTitle:         "요가 기초",
			RegistrationEnabled: true,
			Status:              "open",
			Weekday:             1,
			StartTime:           "09:00",
			EndTime:             "10:00",
		}, nil
	}
	f.offering.ID = id
	return f.offering, nil
}

func TestRegistrationServiceRejectsInvalidInput(t *testing.T) {
	service := NewRegistrationService(&fakeRegistrationRepository{}, fakeMemberLookup{}, fakeCourseOfferingLookup{})

	if _, err := service.Create(context.Background(), RegistrationInput{}); err == nil {
		t.Fatal("Create() error = nil, want validation error")
	}
}

func TestRegistrationServiceCreatesRegistration(t *testing.T) {
	repo := &fakeRegistrationRepository{}
	service := NewRegistrationService(repo, fakeMemberLookup{}, fakeCourseOfferingLookup{})

	registration, err := service.Create(context.Background(), RegistrationInput{MemberID: 1, OfferingID: 2})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if registration.Status != "applied" {
		t.Fatalf("Status = %q, want applied", registration.Status)
	}
	if repo.created.MemberID != 1 || repo.created.OfferingID != 2 {
		t.Fatalf("created = %+v, want member 1 offering 2", repo.created)
	}
}

func TestRegistrationServiceCancelsRegistration(t *testing.T) {
	repo := &fakeRegistrationRepository{}
	service := NewRegistrationService(repo, fakeMemberLookup{}, fakeCourseOfferingLookup{})

	cancelled, err := service.Cancel(context.Background(), 7)
	if err != nil {
		t.Fatalf("Cancel() error = %v", err)
	}
	if cancelled.Status != "cancelled" {
		t.Fatalf("Status = %q, want cancelled", cancelled.Status)
	}
	if repo.cancelledID != 7 {
		t.Fatalf("cancelledID = %d, want 7", repo.cancelledID)
	}
}

func TestRegistrationServiceConfirmsRegistration(t *testing.T) {
	repo := &fakeRegistrationRepository{}
	service := NewRegistrationService(repo, fakeMemberLookup{}, fakeCourseOfferingLookup{})

	confirmed, err := service.Confirm(context.Background(), 7)
	if err != nil {
		t.Fatalf("Confirm() error = %v", err)
	}
	if confirmed.Status != "confirmed" {
		t.Fatalf("Status = %q, want confirmed", confirmed.Status)
	}
	if repo.confirmedID != 7 {
		t.Fatalf("confirmedID = %d, want 7", repo.confirmedID)
	}
}

func TestRegistrationServiceRejectsDuplicateCourse(t *testing.T) {
	repo := &fakeRegistrationRepository{
		activeItems: []domain.RegistrationRuleItem{{OfferingID: 2, TermID: 1, Weekday: 1, StartTime: "09:00", EndTime: "10:00"}},
	}
	service := NewRegistrationService(repo, fakeMemberLookup{}, fakeCourseOfferingLookup{})

	_, err := service.Create(context.Background(), RegistrationInput{MemberID: 1, OfferingID: 2})
	if err == nil {
		t.Fatal("Create() error = nil, want duplicate rule error")
	}
	if err.Error() != "이미 신청한 강좌입니다" {
		t.Fatalf("error = %q, want user-facing duplicate message", err.Error())
	}
	var violation RuleViolation
	if !errors.As(err, &violation) {
		t.Fatal("error is not RuleViolation")
	}
	if violation.Rule != "duplicate_course" {
		t.Fatalf("violation.Rule = %q, want duplicate_course", violation.Rule)
	}
}

func TestRegistrationServiceRejectsMaxRegistrations(t *testing.T) {
	repo := &fakeRegistrationRepository{
		activeItems: []domain.RegistrationRuleItem{{OfferingID: 1, TermID: 1, Weekday: 1, StartTime: "09:00", EndTime: "10:00"}},
	}
	courses := fakeCourseOfferingLookup{offering: domain.CourseOffering{
		TermID:                    1,
		TermStatus:                "open",
		MaxRegistrationsPerMember: 1,
		RegistrationEnabled:       true,
		Status:                    "open",
		Weekday:                   2,
		StartTime:                 "11:00",
		EndTime:                   "12:00",
	}}
	service := NewRegistrationService(repo, fakeMemberLookup{}, courses)

	if _, err := service.Create(context.Background(), RegistrationInput{MemberID: 1, OfferingID: 2}); err == nil {
		t.Fatal("Create() error = nil, want max registrations rule error")
	}
}

func TestRegistrationServiceRejectsTimeConflict(t *testing.T) {
	repo := &fakeRegistrationRepository{
		activeItems: []domain.RegistrationRuleItem{{OfferingID: 1, TermID: 1, Weekday: 1, StartTime: "09:30", EndTime: "10:30"}},
	}
	service := NewRegistrationService(repo, fakeMemberLookup{}, fakeCourseOfferingLookup{})

	if _, err := service.Create(context.Background(), RegistrationInput{MemberID: 1, OfferingID: 2}); err == nil {
		t.Fatal("Create() error = nil, want time conflict rule error")
	}
}

func TestRegistrationServiceRejectsClosedTerm(t *testing.T) {
	courses := fakeCourseOfferingLookup{offering: domain.CourseOffering{
		TermID:              1,
		TermStatus:          "closed",
		RegistrationEnabled: true,
		Status:              "open",
		Weekday:             1,
		StartTime:           "09:00",
		EndTime:             "10:00",
	}}
	service := NewRegistrationService(&fakeRegistrationRepository{}, fakeMemberLookup{}, courses)

	if _, err := service.Create(context.Background(), RegistrationInput{MemberID: 1, OfferingID: 2}); err == nil {
		t.Fatal("Create() error = nil, want closed term rule error")
	}
}
