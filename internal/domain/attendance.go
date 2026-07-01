package domain

type AttendanceSession struct {
	ID          int64
	OfferingID  int64
	TermName    string
	CourseTitle string
	SessionDate string
	Note        string
}

type AttendanceRecord struct {
	ID             int64
	SessionID      int64
	OfferingID     int64
	SessionDate    string
	RegistrationID int64
	MemberID       int64
	MemberNo       string
	MemberName     string
	Status         string
	Note           string
}
