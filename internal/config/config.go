// Package config loads, validates, and persists application settings.
package config

type Config struct {
	App      AppConfig      `json:"app"`
	Server   ServerConfig   `json:"server"`
	Database DatabaseConfig `json:"database"`
	Backup   BackupConfig   `json:"backup"`
	Export   ExportConfig   `json:"export"`
	Logging  LoggingConfig  `json:"logging"`
	UI       UIConfig       `json:"ui"`
}

type AppConfig struct {
	DisplayName string `json:"display_name"`
	EnglishName string `json:"english_name"`
	Mode        string `json:"mode"`
}

type ServerConfig struct {
	Host string `json:"host"`
	Port int    `json:"port"`
}

type DatabaseConfig struct {
	Path string `json:"path"`
}

type BackupConfig struct {
	Path     string `json:"path"`
	KeepDays int    `json:"keep_days"`
}

type ExportConfig struct {
	Path string `json:"path"`
}

type LoggingConfig struct {
	Path  string `json:"path"`
	Level string `json:"level"`
}

type UIConfig struct {
	OpenBrowserOnStart bool `json:"open_browser_on_start"`
}
