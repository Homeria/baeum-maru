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

type fakeLotteryRepository struct {
	candidates []domain.LotteryCandidate
	saved      repository.SaveLotteryRunParams
}

func (f *fakeLotteryRepository) ListCandidatesByOffering(context.Context, int64) ([]domain.LotteryCandidate, error) {
	return f.candidates, nil
}

func (f *fakeLotteryRepository) SaveRun(_ context.Context, params repository.SaveLotteryRunParams) (int64, error) {
	f.saved = params
	return 7, nil
}

type fakeLotteryCourseLookup struct {
	offering domain.CourseOffering
}

func (f fakeLotteryCourseLookup) GetOffering(_ context.Context, id int64) (domain.CourseOffering, error) {
	f.offering.ID = id
	return f.offering, nil
}
