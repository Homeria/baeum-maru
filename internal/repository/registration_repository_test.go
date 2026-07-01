package repository

import (
	"context"
	"testing"
)

func TestRegistrationRepositoryCreateListAndCancel(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	members := NewMemberRepository(db)
	courses := NewCourseRepository(db)
	registrations := NewRegistrationRepository(db)

	member, err := members.Create(ctx, CreateMemberParams{Name: "김배움"})
	if err != nil {
		t.Fatalf("members.Create() error = %v", err)
	}
	offering, err := courses.CreateOffering(ctx, CreateCourseOfferingParams{
		CourseTitle: "요가 기초",
		Capacity:    20,
		Weekday:     1,
		StartTime:   "09:00",
		EndTime:     "10:00",
	})
	if err != nil {
		t.Fatalf("courses.CreateOffering() error = %v", err)
	}

	registration, err := registrations.Create(ctx, CreateRegistrationParams{
		MemberID:   member.ID,
		OfferingID: offering.ID,
	})
	if err != nil {
		t.Fatalf("registrations.Create() error = %v", err)
	}
	if registration.Status != "applied" {
		t.Fatalf("Status = %q, want applied", registration.Status)
	}

	byMember, err := registrations.ListByMember(ctx, member.ID)
	if err != nil {
		t.Fatalf("ListByMember() error = %v", err)
	}
	if len(byMember) != 1 {
		t.Fatalf("len(byMember) = %d, want 1", len(byMember))
	}

	byOffering, err := registrations.ListByOffering(ctx, offering.ID)
	if err != nil {
		t.Fatalf("ListByOffering() error = %v", err)
	}
	if len(byOffering) != 1 {
		t.Fatalf("len(byOffering) = %d, want 1", len(byOffering))
	}

	cancelled, err := registrations.Cancel(ctx, registration.ID)
	if err != nil {
		t.Fatalf("Cancel() error = %v", err)
	}
	if cancelled.Status != "cancelled" {
		t.Fatalf("Status = %q, want cancelled", cancelled.Status)
	}
}

func TestRegistrationRepositoryRejectsDuplicate(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	members := NewMemberRepository(db)
	courses := NewCourseRepository(db)
	registrations := NewRegistrationRepository(db)

	member, err := members.Create(ctx, CreateMemberParams{Name: "김배움"})
	if err != nil {
		t.Fatalf("members.Create() error = %v", err)
	}
	offering, err := courses.CreateOffering(ctx, CreateCourseOfferingParams{
		CourseTitle: "요가 기초",
		Capacity:    20,
		Weekday:     1,
		StartTime:   "09:00",
		EndTime:     "10:00",
	})
	if err != nil {
		t.Fatalf("courses.CreateOffering() error = %v", err)
	}

	_, err = registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offering.ID})
	if err != nil {
		t.Fatalf("first Create() error = %v", err)
	}
	if _, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offering.ID}); err == nil {
		t.Fatal("second Create() error = nil, want duplicate error")
	}
}
