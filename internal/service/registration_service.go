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
	Confirm(context.Context, int64) (domain.Registration, error)
	CancelAndPromote(context.Context, int64) (domain.RegistrationStatusChange, error)
	ListByMember(context.Context, int64) ([]domain.Registration, error)
	ListByOffering(context.Context, int64) ([]domain.Registration, error)
	ListRecent(context.Context, int) ([]domain.Registration, error)
	ListActiveRuleItemsByMember(context.Context, int64) ([]domain.RegistrationRuleItem, error)
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
	rules         RegistrationRuleSet
}

type RegistrationInput struct {
	MemberID   int64
	OfferingID int64
}

func NewRegistrationService(registrations RegistrationRepository, members MemberLookup, courses CourseOfferingLookup) *RegistrationService {
	rules := DefaultRegistrationRuleSet(registrations, courses)
	return &RegistrationService{
		registrations: registrations,
		members:       members,
		courses:       courses,
		rules:         rules,
	}
}

func NewRegistrationServiceWithRules(registrations RegistrationRepository, members MemberLookup, courses CourseOfferingLookup, rules RegistrationRuleSet) *RegistrationService {
	return &RegistrationService{
		registrations: registrations,
		members:       members,
		courses:       courses,
		rules:         rules,
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
	if err := s.rules.Check(ctx, input); err != nil {
		return domain.Registration{}, err
	}

	return s.registrations.Create(ctx, repository.CreateRegistrationParams{
		MemberID:   input.MemberID,
		OfferingID: input.OfferingID,
	})
}

func (s *RegistrationService) Cancel(ctx context.Context, id int64) (domain.Registration, error) {
	change, err := s.CancelWithPromotion(ctx, id)
	if err != nil {
		return domain.Registration{}, err
	}
	return change.Registration, nil
}

func (s *RegistrationService) Confirm(ctx context.Context, id int64) (domain.Registration, error) {
	if id <= 0 {
		return domain.Registration{}, errors.New("registration id is required")
	}
	return s.registrations.Confirm(ctx, id)
}

func (s *RegistrationService) CancelWithPromotion(ctx context.Context, id int64) (domain.RegistrationStatusChange, error) {
	if id <= 0 {
		return domain.RegistrationStatusChange{}, errors.New("registration id is required")
	}
	return s.registrations.CancelAndPromote(ctx, id)
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
