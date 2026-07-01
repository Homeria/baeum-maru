package domain

type Term struct {
	ID     int64
	Name   string
	Status string
}

type Course struct {
	ID           int64
	Title        string
	CategoryID   int64
	CategoryName string
	Description  string
	IsActive     bool
}

type CourseOffering struct {
	ID                  int64
	TermID              int64
	TermName            string
	CourseID            int64
	CourseTitle         string
	CategoryName        string
	InstructorName      string
	ClassroomName       string
	Capacity            int
	RegistrationEnabled bool
	Status              string
	Weekday             int
	StartTime           string
	EndTime             string
	RegistrationCount   int
}
