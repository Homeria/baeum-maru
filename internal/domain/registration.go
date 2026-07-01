package domain

type Registration struct {
	ID          int64
	MemberID    int64
	MemberName  string
	MemberNo    string
	OfferingID  int64
	CourseTitle string
	TermName    string
	Status      string
	CreatedAt   string
	UpdatedAt   string
	CancelledAt string
}

type RegistrationStatusChange struct {
	Registration Registration
	Promoted     *Registration
}

type RegistrationRuleItem struct {
	ID         int64
	MemberID   int64
	OfferingID int64
	TermID     int64
	Status     string
	Weekday    int
	StartTime  string
	EndTime    string
}
