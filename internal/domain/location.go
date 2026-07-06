package domain

const (
	LocationTypeClassroom = "classroom"
	LocationTypeOffice    = "office"
	LocationTypeStorage   = "storage"
	LocationTypeHall      = "hall"
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
