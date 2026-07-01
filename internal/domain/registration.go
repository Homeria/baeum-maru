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
