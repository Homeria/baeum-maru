package domain

type LotteryCandidate struct {
	RegistrationID int64
	MemberID       int64
	MemberName     string
	MemberNo       string
	OfferingID     int64
	Status         string
	CreatedAt      string
}

type LotteryAssignment struct {
	RegistrationID int64
	Result         string
	ResultOrder    int
}

type LotteryRunSummary struct {
	RunID           int64
	TermID          int64
	OfferingID      int64
	CourseTitle     string
	Capacity        int
	Seed            int64
	TotalCount      int
	SelectedCount   int
	WaitlistedCount int
}
