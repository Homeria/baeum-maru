package domain

type BackupFile struct {
	FileName  string
	Path      string
	SizeBytes int64
	CreatedAt string
}

type RestorePlan struct {
	FileName string
	Path     string
}
