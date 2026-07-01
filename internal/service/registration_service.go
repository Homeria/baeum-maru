package service

import (
	"context"
	"errors"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type RegistrationRepository interface {
	Create(context.Context, repository.CreateRegistrationParams) (domain.Registration, error)
	Cancel(context.Context, int64) (domain.Registration, error)
	ListByMember(context.Context, int64) ([]domain.Registration, error)
	ListByOffering(context.Context, int64) ([]domain.Registration, error)
	ListRecent(context.Context, int) ([]domain.Registration, error)
}

type MemberLookup interface {
	Get(context.Context, int64) (domain.Member, error)
}

type CourseOfferingLookup interface {
	GetOffering(context.Context, int64) (domain.CourseOffering, error)
}

type RegistrationService struct {
	registrations RegistrationRepository
	members       MemberLookup
	courses       CourseOfferingLookup
}

type RegistrationInput struct {
	MemberID   int64
	OfferingID int64
}

func NewRegistrationService(registrations RegistrationRepository, members MemberLookup, courses CourseOfferingLookup) *RegistrationService {
	return &RegistrationService{
		registrations: registrations,
		members:       members,
		courses:       courses,
	}
}

func (s *RegistrationService) Create(ctx context.Context, input RegistrationInput) (domain.Registration, error) {
	if input.MemberID <= 0 {
		return domain.Registration{}, errors.New("member id is required")
	}
	if input.OfferingID <= 0 {
		return domain.Registration{}, errors.New("course offering id is required")
	}
	if _, err := s.members.Get(ctx, input.MemberID); err != nil {
		return domain.Registration{}, err
	}
	if _, err := s.courses.GetOffering(ctx, input.OfferingID); err != nil {
		return domain.Registration{}, err
	}

	return s.registrations.Create(ctx, repository.CreateRegistrationParams{
		MemberID:   input.MemberID,
		OfferingID: input.OfferingID,
	})
}

func (s *RegistrationService) Cancel(ctx context.Context, id int64) (domain.Registration, error) {
	if id <= 0 {
		return domain.Registration{}, errors.New("registration id is required")
	}
	return s.registrations.Cancel(ctx, id)
}

func (s *RegistrationService) ListByMember(ctx context.Context, memberID int64) ([]domain.Registration, error) {
	if memberID <= 0 {
		return nil, nil
	}
	return s.registrations.ListByMember(ctx, memberID)
}

func (s *RegistrationService) ListByOffering(ctx context.Context, offeringID int64) ([]domain.Registration, error) {
	if offeringID <= 0 {
		return nil, nil
	}
	return s.registrations.ListByOffering(ctx, offeringID)
}

func (s *RegistrationService) ListRecent(ctx context.Context, limit int) ([]domain.Registration, error) {
	return s.registrations.ListRecent(ctx, limit)
}
