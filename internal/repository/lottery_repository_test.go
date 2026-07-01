package repository

import (
	"context"
	"database/sql"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
)

func TestLotteryRepositoryListsCandidatesAndSavesRun(t *testing.T) {
	ctx := context.Background()
	db := newTestDB(t)
	members := NewMemberRepository(db)
	courses := NewCourseRepository(db)
	registrations := NewRegistrationRepository(db)
	lotteries := NewLotteryRepository(db)

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
	first := createLotteryRegistration(t, ctx, members, registrations, offering.ID, "김배움")
	second := createLotteryRegistration(t, ctx, members, registrations, offering.ID, "이마루")

	candidates, err := lotteries.ListCandidatesByOffering(ctx, offering.ID)
	if err != nil {
		t.Fatalf("ListCandidatesByOffering() error = %v", err)
	}
	if len(candidates) != 2 {
		t.Fatalf("len(candidates) = %d, want 2", len(candidates))
	}

	runID, err := lotteries.SaveRun(ctx, SaveLotteryRunParams{
		TermID: offering.TermID,
		Seed:   100,
		Assignments: []domain.LotteryAssignment{
			{RegistrationID: first.ID, Result: "selected", ResultOrder: 1},
			{RegistrationID: second.ID, Result: "waitlisted", ResultOrder: 2},
		},
	})
	if err != nil {
		t.Fatalf("SaveRun() error = %v", err)
	}
	if runID == 0 {
		t.Fatal("runID = 0, want created run")
	}

	assertRegistrationStatus(t, db, first.ID, "selected")
	assertRegistrationStatus(t, db, second.ID, "waitlisted")
	assertLotteryResultCount(t, db, runID, 2)

	runs, err := lotteries.ListRuns(ctx, 10)
	if err != nil {
		t.Fatalf("ListRuns() error = %v", err)
	}
	if len(runs) != 1 {
		t.Fatalf("len(runs) = %d, want 1", len(runs))
	}
	if runs[0].ID != runID || runs[0].SelectedCount != 1 || runs[0].WaitlistedCount != 1 {
		t.Fatalf("run = %+v, want saved run counts", runs[0])
	}

	results, err := lotteries.ListResultsByRun(ctx, runID)
	if err != nil {
		t.Fatalf("ListResultsByRun() error = %v", err)
	}
	if len(results) != 2 {
		t.Fatalf("len(results) = %d, want 2", len(results))
	}
	if results[0].Result != "selected" || results[0].MemberName != "김배움" {
		t.Fatalf("first result = %+v, want selected 김배움", results[0])
	}
}

func createLotteryRegistration(t *testing.T, ctx context.Context, members *MemberRepository, registrations *RegistrationRepository, offeringID int64, name string) domain.Registration {
	t.Helper()

	member, err := members.Create(ctx, CreateMemberParams{Name: name})
	if err != nil {
		t.Fatalf("members.Create() error = %v", err)
	}
	registration, err := registrations.Create(ctx, CreateRegistrationParams{MemberID: member.ID, OfferingID: offeringID})
	if err != nil {
		t.Fatalf("registrations.Create() error = %v", err)
	}
	return registration
}

func assertRegistrationStatus(t *testing.T, db *sql.DB, registrationID int64, want string) {
	t.Helper()

	var got string
	if err := db.QueryRow("SELECT status FROM registrations WHERE id = ?;", registrationID).Scan(&got); err != nil {
		t.Fatalf("read registration status: %v", err)
	}
	if got != want {
		t.Fatalf("registration %d status = %q, want %q", registrationID, got, want)
	}
}

func assertLotteryResultCount(t *testing.T, db *sql.DB, runID int64, want int) {
	t.Helper()

	var got int
	if err := db.QueryRow("SELECT COUNT(*) FROM lottery_results WHERE lottery_run_id = ?;", runID).Scan(&got); err != nil {
		t.Fatalf("read lottery result count: %v", err)
	}
	if got != want {
		t.Fatalf("lottery result count = %d, want %d", got, want)
	}
}
