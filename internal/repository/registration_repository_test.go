package repository

import (
	"context"
	"database/sql"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
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

func TestRegistrationRepositoryReactivatesCancelledRegistration(t *testing.T) {
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

	first, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offering.ID})
	if err != nil {
		t.Fatalf("first Create() error = %v", err)
	}
	if _, err := registrations.Cancel(ctx, first.ID); err != nil {
		t.Fatalf("Cancel() error = %v", err)
	}

	second, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offering.ID})
	if err != nil {
		t.Fatalf("second Create() error = %v", err)
	}
	if second.ID != first.ID {
		t.Fatalf("second.ID = %d, want existing ID %d", second.ID, first.ID)
	}
	if second.Status != "applied" {
		t.Fatalf("second.Status = %q, want applied", second.Status)
	}
}

func TestRegistrationRepositoryConfirmsSelectedRegistration(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	members := NewMemberRepository(db)
	courses := NewCourseRepository(db)
	registrations := NewRegistrationRepository(db)

	registration := createRegistrationForStatusTest(t, ctx, members, courses, registrations, "김배움", "selected")

	confirmed, err := registrations.Confirm(ctx, registration.ID)
	if err != nil {
		t.Fatalf("Confirm() error = %v", err)
	}
	if confirmed.Status != "confirmed" {
		t.Fatalf("confirmed.Status = %q, want confirmed", confirmed.Status)
	}
	assertRegistrationHistory(t, db, registration.ID, "selected", "confirmed")
}

func TestRegistrationRepositoryCancelsAndPromotesWaitlisted(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	members := NewMemberRepository(db)
	courses := NewCourseRepository(db)
	registrations := NewRegistrationRepository(db)

	offering, err := courses.CreateOffering(ctx, CreateCourseOfferingParams{
		CourseTitle: "요가 기초",
		Capacity:    1,
		Weekday:     1,
		StartTime:   "09:00",
		EndTime:     "10:00",
	})
	if err != nil {
		t.Fatalf("courses.CreateOffering() error = %v", err)
	}
	selected := createRegistrationWithOfferingForStatusTest(t, ctx, members, registrations, offering.ID, "김배움", "selected")
	waitlisted := createRegistrationWithOfferingForStatusTest(t, ctx, members, registrations, offering.ID, "이마루", "waitlisted")

	change, err := registrations.CancelAndPromote(ctx, selected.ID)
	if err != nil {
		t.Fatalf("CancelAndPromote() error = %v", err)
	}
	if change.Registration.Status != "cancelled" {
		t.Fatalf("cancelled.Status = %q, want cancelled", change.Registration.Status)
	}
	if change.Promoted == nil {
		t.Fatal("Promoted = nil, want waitlisted registration")
	}
	if change.Promoted.ID != waitlisted.ID || change.Promoted.Status != "selected" {
		t.Fatalf("Promoted = %+v, want waitlisted promoted to selected", change.Promoted)
	}
	assertRegistrationHistory(t, db, selected.ID, "selected", "cancelled")
	assertRegistrationHistory(t, db, waitlisted.ID, "waitlisted", "selected")
}

func TestRegistrationRepositoryListsRuleItems(t *testing.T) {
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
	if _, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offering.ID}); err != nil {
		t.Fatalf("registrations.Create() error = %v", err)
	}

	items, err := registrations.ListActiveRuleItemsByMember(ctx, member.ID)
	if err != nil {
		t.Fatalf("ListActiveRuleItemsByMember() error = %v", err)
	}
	if len(items) != 1 {
		t.Fatalf("len(items) = %d, want 1", len(items))
	}
	if items[0].Weekday != 1 || items[0].StartTime != "09:00" || items[0].EndTime != "10:00" {
		t.Fatalf("item = %+v, want meeting data", items[0])
	}
}

func createRegistrationForStatusTest(t *testing.T, ctx context.Context, members *MemberRepository, courses *CourseRepository, registrations *RegistrationRepository, name string, status string) domain.Registration {
	t.Helper()

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
	return createRegistrationWithOfferingForStatusTest(t, ctx, members, registrations, offering.ID, name, status)
}

func createRegistrationWithOfferingForStatusTest(t *testing.T, ctx context.Context, members *MemberRepository, registrations *RegistrationRepository, offeringID int64, name string, status string) domain.Registration {
	t.Helper()

	member, err := members.Create(ctx, CreateMemberParams{Name: name})
	if err != nil {
		t.Fatalf("members.Create() error = %v", err)
	}
	registration, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offeringID})
	if err != nil {
		t.Fatalf("registrations.Create() error = %v", err)
	}
	if _, err := registrations.db.ExecContext(ctx, "UPDATE registrations SET status = ? WHERE id = ?;", status, registration.ID); err != nil {
		t.Fatalf("set registration status: %v", err)
	}
	registration.Status = status
	return registration
}

func assertRegistrationHistory(t *testing.T, db interface {
	QueryRowContext(context.Context, string, ...any) *sql.Row
}, registrationID int64, fromStatus string, toStatus string) {
	t.Helper()

	var count int
	if err := db.QueryRowContext(context.Background(), `
SELECT COUNT(*)
FROM registration_status_history
WHERE registration_id = ? AND from_status = ? AND to_status = ?;
`, registrationID, fromStatus, toStatus).Scan(&count); err != nil {
		t.Fatalf("read registration history: %v", err)
	}
	if count == 0 {
		t.Fatalf("history %d %s -> %s count = 0, want recorded", registrationID, fromStatus, toStatus)
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
