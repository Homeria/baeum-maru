package service

import (
	"context"
	"testing"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

func TestLotteryServiceSelectsAllWhenWithinCapacity(t *testing.T) {
	lotteries := &fakeLotteryRepository{
		candidates: []domain.LotteryCandidate{
			{RegistrationID: 1},
			{RegistrationID: 2},
		},
	}
	service := NewLotteryService(lotteries, fakeLotteryCourseLookup{offering: domain.CourseOffering{
		ID:          10,
		TermID:      3,
		CourseTitle: "요가 기초",
		Capacity:    3,
	}})
	service.newSeed = func() int64 { return 100 }

	summary, err := service.RunOfferingLottery(context.Background(), 10)
	if err != nil {
		t.Fatalf("RunOfferingLottery() error = %v", err)
	}
	if summary.SelectedCount != 2 || summary.WaitlistedCount != 0 {
		t.Fatalf("summary = %+v, want all selected", summary)
	}
	for _, assignment := range lotteries.saved.Assignments {
		if assignment.Result != "selected" {
			t.Fatalf("assignment.Result = %q, want selected", assignment.Result)
		}
	}
}

func TestLotteryServiceWaitlistsOverCapacity(t *testing.T) {
	lotteries := &fakeLotteryRepository{
		candidates: []domain.LotteryCandidate{
			{RegistrationID: 1},
			{RegistrationID: 2},
			{RegistrationID: 3},
		},
	}
	service := NewLotteryService(lotteries, fakeLotteryCourseLookup{offering: domain.CourseOffering{
		ID:          10,
		TermID:      3,
		CourseTitle: "요가 기초",
		Capacity:    1,
	}})
	service.newSeed = func() int64 { return 100 }

	summary, err := service.RunOfferingLottery(context.Background(), 10)
	if err != nil {
		t.Fatalf("RunOfferingLottery() error = %v", err)
	}
	if summary.SelectedCount != 1 || summary.WaitlistedCount != 2 {
		t.Fatalf("summary = %+v, want 1 selected 2 waitlisted", summary)
	}
	if len(lotteries.saved.Assignments) != 3 {
		t.Fatalf("len(assignments) = %d, want 3", len(lotteries.saved.Assignments))
	}
}

func TestLotteryServiceRequiresForceForRerun(t *testing.T) {
	lotteries := &fakeLotteryRepository{
		existingRun: domain.LotteryRun{ID: 3, OfferingID: 10, CourseTitle: "요가 기초"},
	}
	service := NewLotteryService(lotteries, fakeLotteryCourseLookup{offering: domain.CourseOffering{
		ID:          10,
		TermID:      3,
		CourseTitle: "요가 기초",
		Capacity:    1,
	}})

	_, err := service.RunOfferingLottery(context.Background(), 10)
	if err == nil {
		t.Fatal("RunOfferingLottery() error = nil, want rerun required error")
	}
	if _, ok := err.(*LotteryRerunRequiredError); !ok {
		t.Fatalf("error = %T, want *LotteryRerunRequiredError", err)
	}
	if len(lotteries.saved.Assignments) != 0 {
		t.Fatalf("saved assignments = %d, want none", len(lotteries.saved.Assignments))
	}
}

func TestLotteryServiceAllowsForcedRerun(t *testing.T) {
	lotteries := &fakeLotteryRepository{
		candidates:  []domain.LotteryCandidate{{RegistrationID: 1}},
		existingRun: domain.LotteryRun{ID: 3, OfferingID: 10, CourseTitle: "요가 기초"},
		nextRunID:   8,
	}
	service := NewLotteryService(lotteries, fakeLotteryCourseLookup{offering: domain.CourseOffering{
		ID:          10,
		TermID:      3,
		CourseTitle: "요가 기초",
		Capacity:    1,
	}})
	service.newSeed = func() int64 { return 100 }

	summary, err := service.RunOfferingLottery(context.Background(), 10, LotteryRunOptions{ForceRerun: true})
	if err != nil {
		t.Fatalf("RunOfferingLottery() error = %v", err)
	}
	if !summary.Rerun || summary.PreviousRunID != 3 || summary.RunID != 8 {
		t.Fatalf("summary = %+v, want rerun with previous run id", summary)
	}
}

type fakeLotteryRepository struct {
	candidates  []domain.LotteryCandidate
	existingRun domain.LotteryRun
	nextRunID   int64
	saved       repository.SaveLotteryRunParams
}

func (f *fakeLotteryRepository) ListCandidatesByOffering(context.Context, int64) ([]domain.LotteryCandidate, error) {
	return f.candidates, nil
}

func (f *fakeLotteryRepository) ListRuns(context.Context, int) ([]domain.LotteryRun, error) {
	return []domain.LotteryRun{{ID: 7}}, nil
}

func (f *fakeLotteryRepository) ListResultsByRun(context.Context, int64) ([]domain.LotteryResultRow, error) {
	return []domain.LotteryResultRow{{RunID: 7}}, nil
}

func (f *fakeLotteryRepository) LatestRunByOffering(context.Context, int64) (domain.LotteryRun, bool, error) {
	if f.existingRun.ID == 0 {
		return domain.LotteryRun{}, false, nil
	}
	return f.existingRun, true, nil
}

func (f *fakeLotteryRepository) SaveRun(_ context.Context, params repository.SaveLotteryRunParams) (int64, error) {
	f.saved = params
	if f.nextRunID != 0 {
		return f.nextRunID, nil
	}
	return 7, nil
}

type fakeLotteryCourseLookup struct {
	offering domain.CourseOffering
}

func (f fakeLotteryCourseLookup) GetOffering(_ context.Context, id int64) (domain.CourseOffering, error) {
	f.offering.ID = id
	return f.offering, nil
}
