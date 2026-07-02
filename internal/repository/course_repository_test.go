package repository

import (
	"context"
	"testing"
)

func TestCourseRepositoryCreateAndListOffering(t *testing.T) {
	ctx := context.Background()
	repo := NewCourseRepository(newTestDB(t))

	offering, err := repo.CreateOffering(ctx, CreateCourseOfferingParams{
		TermName:       "2026년 여름학기",
		CategoryName:   "건강",
		CourseTitle:    "요가 기초",
		InstructorName: "홍강사",
		ClassroomName:  "101호",
		Capacity:       20,
		Weekday:        1,
		StartTime:      "09:00",
		EndTime:        "10:00",
	})
	if err != nil {
		t.Fatalf("CreateOffering() error = %v", err)
	}
	if offering.CourseTitle != "요가 기초" {
		t.Fatalf("CourseTitle = %q, want 요가 기초", offering.CourseTitle)
	}

	offerings, err := repo.ListOfferings(ctx, 10)
	if err != nil {
		t.Fatalf("ListOfferings() error = %v", err)
	}
	if len(offerings) != 1 {
		t.Fatalf("len(offerings) = %d, want 1", len(offerings))
	}
	if offerings[0].Capacity != 20 {
		t.Fatalf("Capacity = %d, want 20", offerings[0].Capacity)
	}
}

func TestCourseRepositoryUpdatesOffering(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	repo := NewCourseRepository(db)

	offering, err := repo.CreateOffering(ctx, CreateCourseOfferingParams{
		TermName:       "2026년 여름학기",
		CategoryName:   "건강",
		CourseTitle:    "요가 기초",
		InstructorName: "홍강사",
		ClassroomName:  "101호",
		Capacity:       20,
		Weekday:        1,
		StartTime:      "09:00",
		EndTime:        "10:00",
		Note:           "첫 수업",
	})
	if err != nil {
		t.Fatalf("CreateOffering() error = %v", err)
	}

	updated, err := repo.UpdateOffering(ctx, UpdateCourseOfferingParams{
		ID:             offering.ID,
		TermName:       "2026년 가을학기",
		CategoryName:   "음악",
		CourseTitle:    "합창 기초",
		InstructorName: "김강사",
		ClassroomName:  "202호",
		Capacity:       15,
		Weekday:        3,
		StartTime:      "13:00",
		EndTime:        "14:30",
		Note:           "수정됨",
	})
	if err != nil {
		t.Fatalf("UpdateOffering() error = %v", err)
	}
	if updated.CourseTitle != "합창 기초" {
		t.Fatalf("CourseTitle = %q, want 합창 기초", updated.CourseTitle)
	}
	if updated.TermName != "2026년 가을학기" || updated.CategoryName != "음악" || updated.Capacity != 15 {
		t.Fatalf("updated = %+v, want changed term/category/capacity", updated)
	}
	if updated.Weekday != 3 || updated.StartTime != "13:00" || updated.EndTime != "14:30" {
		t.Fatalf("updated meeting = %+v, want Wednesday 13:00-14:30", updated)
	}
	if updated.Note != "수정됨" {
		t.Fatalf("Note = %q, want 수정됨", updated.Note)
	}
}

func TestCourseRepositoryRejectsCapacityBelowActiveRegistrations(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	courses := NewCourseRepository(db)
	members := NewMemberRepository(db)
	registrations := NewRegistrationRepository(db)

	offering, err := courses.CreateOffering(ctx, CreateCourseOfferingParams{
		CourseTitle: "요가 기초",
		Capacity:    2,
		Weekday:     1,
		StartTime:   "09:00",
		EndTime:     "10:00",
	})
	if err != nil {
		t.Fatalf("CreateOffering() error = %v", err)
	}
	member, err := members.Create(ctx, CreateMemberParams{Name: "김배움"})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if _, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offering.ID}); err != nil {
		t.Fatalf("registrations.Create() error = %v", err)
	}

	if _, err := courses.UpdateOffering(ctx, UpdateCourseOfferingParams{
		ID:          offering.ID,
		CourseTitle: "요가 기초",
		Capacity:    0,
		Weekday:     1,
		StartTime:   "09:00",
		EndTime:     "10:00",
	}); err == nil {
		t.Fatal("UpdateOffering() error = nil, want capacity error")
	}
}
