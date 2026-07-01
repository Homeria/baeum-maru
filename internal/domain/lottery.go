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
	Rerun           bool
	PreviousRunID   int64
}

type LotteryRun struct {
	ID              int64
	TermID          int64
	TermName        string
	Seed            int64
	Status          string
	StartedAt       string
	CompletedAt     string
	OfferingID      int64
	CourseTitle     string
	TotalCount      int
	SelectedCount   int
	WaitlistedCount int
}

type LotteryResultRow struct {
	RunID                 int64
	Seed                  int64
	CompletedAt           string
	OfferingID            int64
	CourseTitle           string
	TermName              string
	Result                string
	ResultOrder           int
	RegistrationID        int64
	MemberID              int64
	MemberNo              string
	MemberName            string
	RegistrationCreatedAt string
}
