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
