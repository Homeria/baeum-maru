package service

import (
	"context"
	"errors"
	"fmt"

	"github.com/Homeria/baeum-maru/internal/domain"
)

type RegistrationRule interface {
	Name() string
	Check(context.Context, RegistrationInput) error
}

type RegistrationRuleSet []RegistrationRule

type RuleViolation struct {
	Rule string
	Err  error
}

func (e RuleViolation) Error() string {
	if e.Err == nil {
		return ""
	}
	return e.Err.Error()
}

func (e RuleViolation) Unwrap() error {
	return e.Err
}

func (rules RegistrationRuleSet) Check(ctx context.Context, input RegistrationInput) error {
	for _, rule := range rules {
		if err := rule.Check(ctx, input); err != nil {
			return RuleViolation{Rule: rule.Name(), Err: err}
		}
	}
	return nil
}

type RegistrationRuleItemLookup interface {
	ListActiveRuleItemsByMember(context.Context, int64) ([]domain.RegistrationRuleItem, error)
}

func DefaultRegistrationRuleSet(registrations RegistrationRuleItemLookup, courses CourseOfferingLookup) RegistrationRuleSet {
	return RegistrationRuleSet{
		TermOpenRule{courses: courses},
		DuplicateCourseRule{registrations: registrations},
		MaxRegistrationsPerMemberRule{registrations: registrations, courses: courses},
		TimeConflictRule{registrations: registrations, courses: courses},
	}
}

type TermOpenRule struct {
	courses CourseOfferingLookup
}

func (TermOpenRule) Name() string {
	return "term_open"
}

func (r TermOpenRule) Check(ctx context.Context, input RegistrationInput) error {
	offering, err := r.courses.GetOffering(ctx, input.OfferingID)
	if err != nil {
		return err
	}
	if offering.TermStatus != "open" {
		return errors.New("접수 중인 회차가 아닙니다")
	}
	if offering.Status != "open" {
		return errors.New("접수 가능한 강좌가 아닙니다")
	}
	if !offering.RegistrationEnabled {
		return errors.New("접수가 비활성화된 강좌입니다")
	}
	return nil
}

type DuplicateCourseRule struct {
	registrations RegistrationRuleItemLookup
}

func (DuplicateCourseRule) Name() string {
	return "duplicate_course"
}

func (r DuplicateCourseRule) Check(ctx context.Context, input RegistrationInput) error {
	items, err := r.registrations.ListActiveRuleItemsByMember(ctx, input.MemberID)
	if err != nil {
		return err
	}
	for _, item := range items {
		if item.OfferingID == input.OfferingID {
			return errors.New("이미 신청한 강좌입니다")
		}
	}
	return nil
}

type MaxRegistrationsPerMemberRule struct {
	registrations RegistrationRuleItemLookup
	courses       CourseOfferingLookup
}

func (MaxRegistrationsPerMemberRule) Name() string {
	return "max_registrations_per_member"
}

func (r MaxRegistrationsPerMemberRule) Check(ctx context.Context, input RegistrationInput) error {
	offering, err := r.courses.GetOffering(ctx, input.OfferingID)
	if err != nil {
		return err
	}
	if offering.MaxRegistrationsPerMember <= 0 {
		return nil
	}
	items, err := r.registrations.ListActiveRuleItemsByMember(ctx, input.MemberID)
	if err != nil {
		return err
	}

	count := 0
	seenOfferings := make(map[int64]struct{})
	for _, item := range items {
		if item.TermID != offering.TermID {
			continue
		}
		if _, exists := seenOfferings[item.OfferingID]; exists {
			continue
		}
		seenOfferings[item.OfferingID] = struct{}{}
		count++
	}
	if count >= offering.MaxRegistrationsPerMember {
		return fmt.Errorf("1인 최대 신청 강좌 수 %d개를 초과할 수 없습니다", offering.MaxRegistrationsPerMember)
	}
	return nil
}

type TimeConflictRule struct {
	registrations RegistrationRuleItemLookup
	courses       CourseOfferingLookup
}

func (TimeConflictRule) Name() string {
	return "time_conflict"
}

func (r TimeConflictRule) Check(ctx context.Context, input RegistrationInput) error {
	offering, err := r.courses.GetOffering(ctx, input.OfferingID)
	if err != nil {
		return err
	}
	items, err := r.registrations.ListActiveRuleItemsByMember(ctx, input.MemberID)
	if err != nil {
		return err
	}

	for _, schedule := range offeringRuleSchedules(offering) {
		for _, item := range items {
			if item.TermID != offering.TermID {
				continue
			}
			if item.Weekday != schedule.Weekday {
				continue
			}
			if item.StartTime < schedule.EndTime && schedule.StartTime < item.EndTime {
				return errors.New("같은 시간대의 다른 강좌를 이미 신청했습니다")
			}
		}
	}
	return nil
}

func offeringRuleSchedules(offering domain.CourseOffering) []domain.CourseSchedule {
	if len(offering.Schedules) > 0 {
		return offering.Schedules
	}
	return []domain.CourseSchedule{{
		Weekday:   offering.Weekday,
		StartTime: offering.StartTime,
		EndTime:   offering.EndTime,
	}}
}
