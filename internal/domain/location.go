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
	Name        string
	Building    string
	Floor       string
	Type        string
	IsClassroom bool
	IsActive    bool
	Note        string
	CreatedAt   string
	UpdatedAt   string
}
