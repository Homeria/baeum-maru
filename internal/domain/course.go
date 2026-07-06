package domain

type Term struct {
	ID     int64
	Name   string
	Status string
}

type Course struct {
	ID           int64
	Name         string
	CategoryID   int64
	CategoryName string
	Description  string
	IsActive     bool
}

type CourseSchedule struct {
	ID         int64
	Weekday    int
	TimeSlotID int64
	TimeSlot   string
	StartTime  string
	EndTime    string
}

type CourseOffering struct {
	ID                        int64
	TermID                    int64
	TermName                  string
	TermStatus                string
	MaxRegistrationsPerMember int
	CourseID                  int64
	CourseTitle               string
	CategoryName              string
	DisplayName               string
	LevelLabel                string
	SectionLabel              string
	InstructorName            string
	ClassroomName             string
	LocationID                int64
	LocationName              string
	FloorLabel                string
	Capacity                  int
	CapacityType              string
	CapacityTotal             int
	MaleCapacity              int
	FemaleCapacity            int
	CapacityLabel             string
	RegistrationEnabled       bool
	Status                    string
	Weekday                   int
	StartTime                 string
	EndTime                   string
	ScheduleText              string
	Schedules                 []CourseSchedule
	Note                      string
	RegistrationCount         int
}
