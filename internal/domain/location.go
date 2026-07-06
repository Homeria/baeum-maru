package domain

const (
	LocationTypeClassroom = "classroom"
	LocationTypeOffice    = "office"
	LocationTypeStorage   = "storage"
	LocationTypeHall      = "event"
	LocationTypeReception = "reception"
	LocationTypeOther     = "other"
)

type Location struct {
	ID          int64
	BuildingID  int64
	Name        string
	Building    string
	Floor       string
	FloorLabel  string
	Type        string
	IsClassroom bool
	IsActive    bool
	Note        string
	Roles       []string
	CreatedAt   string
	UpdatedAt   string
}

type Building struct {
	ID       int64
	Name     string
	IsActive bool
}

type BuildingFloor struct {
	ID         int64
	BuildingID int64
	Building   string
	Label      string
	IsActive   bool
	SortOrder  int
}

type LocationRole struct {
	ID        int64
	Name      string
	IsActive  bool
	SortOrder int
}
