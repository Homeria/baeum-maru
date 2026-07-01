package repository

import (
	"context"
	"testing"
)

func TestAttendanceRepositoryCreatesSessionAndRecordsAttendance(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	members := NewMemberRepository(db)
	courses := NewCourseRepository(db)
	registrations := NewRegistrationRepository(db)
	attendance := NewAttendanceRepository(db)

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
	registration, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offering.ID})
	if err != nil {
		t.Fatalf("registrations.Create() error = %v", err)
	}
	if _, err := db.ExecContext(ctx, "UPDATE registrations SET status = 'confirmed' WHERE id = ?;", registration.ID); err != nil {
		t.Fatalf("confirm registration: %v", err)
	}

	session, err := attendance.CreateSession(ctx, CreateAttendanceSessionParams{
		OfferingID:  offering.ID,
		SessionDate: "2026-07-01",
		Note:        "1회차",
	})
	if err != nil {
		t.Fatalf("CreateSession() error = %v", err)
	}
	if session.CourseTitle != "요가 기초" {
		t.Fatalf("CourseTitle = %q, want 요가 기초", session.CourseTitle)
	}

	confirmed, err := attendance.ListConfirmedByOffering(ctx, offering.ID)
	if err != nil {
		t.Fatalf("ListConfirmedByOffering() error = %v", err)
	}
	if len(confirmed) != 1 {
		t.Fatalf("len(confirmed) = %d, want 1", len(confirmed))
	}

	emptyRecords, err := attendance.ListRecordsBySession(ctx, session.ID)
	if err != nil {
		t.Fatalf("ListRecordsBySession() error = %v", err)
	}
	if len(emptyRecords) != 1 || emptyRecords[0].Status != "" {
		t.Fatalf("emptyRecords = %+v, want one unmarked participant", emptyRecords)
	}

	record, err := attendance.SaveRecord(ctx, SaveAttendanceRecordParams{
		SessionID:      session.ID,
		RegistrationID: registration.ID,
		Status:         "present",
		Note:           "출석",
	})
	if err != nil {
		t.Fatalf("SaveRecord() error = %v", err)
	}
	if record.Status != "present" {
		t.Fatalf("record.Status = %q, want present", record.Status)
	}

	records, err := attendance.ListRecordsBySession(ctx, session.ID)
	if err != nil {
		t.Fatalf("ListRecordsBySession() after save error = %v", err)
	}
	if records[0].Status != "present" || records[0].MemberName != "김배움" {
		t.Fatalf("records[0] = %+v, want present 김배움", records[0])
	}
}
