package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type fakeRegistrationRepository struct {
	created     repository.CreateRegistrationParams
	cancelledID int64
}

func (f *fakeRegistrationRepository) Create(_ context.Context, params repository.CreateRegistrationParams) (domain.Registration, error) {
	f.created = params
	return domain.Registration{ID: 1, MemberID: params.MemberID, OfferingID: params.OfferingID, Status: "applied"}, nil
}

func (f *fakeRegistrationRepository) Cancel(_ context.Context, id int64) (domain.Registration, error) {
	f.cancelledID = id
	return domain.Registration{ID: id, Status: "cancelled"}, nil
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

type fakeMemberLookup struct{}

func (fakeMemberLookup) Get(_ context.Context, id int64) (domain.Member, error) {
	return domain.Member{ID: id, Name: "김배움"}, nil
}

type fakeCourseOfferingLookup struct{}

func (fakeCourseOfferingLookup) GetOffering(_ context.Context, id int64) (domain.CourseOffering, error) {
	return domain.CourseOffering{ID: id, CourseTitle: "요가 기초"}, nil
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
