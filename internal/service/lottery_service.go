package service

import (
	"context"
	"errors"
	"math/rand"
	"time"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/repository"
)

type LotteryRepository interface {
	ListCandidatesByOffering(context.Context, int64) ([]domain.LotteryCandidate, error)
	SaveRun(context.Context, repository.SaveLotteryRunParams) (int64, error)
}

type LotteryCourseLookup interface {
	GetOffering(context.Context, int64) (domain.CourseOffering, error)
}

type LotteryService struct {
	lotteries LotteryRepository
	courses   LotteryCourseLookup
	newSeed   func() int64
}

func NewLotteryService(lotteries LotteryRepository, courses LotteryCourseLookup) *LotteryService {
	return &LotteryService{
		lotteries: lotteries,
		courses:   courses,
		newSeed:   func() int64 { return time.Now().UnixNano() },
	}
}

func (s *LotteryService) RunOfferingLottery(ctx context.Context, offeringID int64) (domain.LotteryRunSummary, error) {
	if offeringID <= 0 {
		return domain.LotteryRunSummary{}, errors.New("course offering id is required")
	}

	offering, err := s.courses.GetOffering(ctx, offeringID)
	if err != nil {
		return domain.LotteryRunSummary{}, err
	}
	candidates, err := s.lotteries.ListCandidatesByOffering(ctx, offeringID)
	if err != nil {
		return domain.LotteryRunSummary{}, err
	}

	seed := s.newSeed()
	assignments := assignLotteryResults(candidates, offering.Capacity, seed)
	runID, err := s.lotteries.SaveRun(ctx, repository.SaveLotteryRunParams{
		TermID:      offering.TermID,
		Seed:        seed,
		Assignments: assignments,
	})
	if err != nil {
		return domain.LotteryRunSummary{}, err
	}

	return summarizeLotteryRun(runID, offering, seed, assignments), nil
}

func assignLotteryResults(candidates []domain.LotteryCandidate, capacity int, seed int64) []domain.LotteryAssignment {
	if capacity < 0 {
		capacity = 0
	}
	assignments := make([]domain.LotteryAssignment, 0, len(candidates))
	if len(candidates) == 0 {
		return assignments
	}

	ordered := append([]domain.LotteryCandidate(nil), candidates...)
	if len(candidates) > capacity {
		random := rand.New(rand.NewSource(seed))
		random.Shuffle(len(ordered), func(i, j int) {
			ordered[i], ordered[j] = ordered[j], ordered[i]
		})
	}

	for i, candidate := range ordered {
		result := "selected"
		if i >= capacity {
			result = "waitlisted"
		}
		assignments = append(assignments, domain.LotteryAssignment{
			RegistrationID: candidate.RegistrationID,
			Result:         result,
			ResultOrder:    i + 1,
		})
	}
	return assignments
}

func summarizeLotteryRun(runID int64, offering domain.CourseOffering, seed int64, assignments []domain.LotteryAssignment) domain.LotteryRunSummary {
	summary := domain.LotteryRunSummary{
		RunID:       runID,
		TermID:      offering.TermID,
		OfferingID:  offering.ID,
		CourseTitle: offering.CourseTitle,
		Capacity:    offering.Capacity,
		Seed:        seed,
		TotalCount:  len(assignments),
	}
	for _, assignment := range assignments {
		switch assignment.Result {
		case "selected":
			summary.SelectedCount++
		case "waitlisted":
			summary.WaitlistedCount++
		}
	}
	return summary
}
