// Package domain defines core business entities and status values.
package domain

type Member struct {
	ID         int64
	MemberNo   string
	Name       string
	GenderCode string
	BirthDate  string
	Phone      string
	Note       string
	CreatedAt  string
	UpdatedAt  string
}
