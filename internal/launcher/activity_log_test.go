package launcher

import (
	"reflect"
	"testing"
	"time"
)

func TestActivityLogKeepsNewestEntriesWithinLimit(t *testing.T) {
	now := time.Date(2026, 7, 10, 9, 0, 0, 0, time.Local)
	log := newActivityLog(2, func() time.Time {
		current := now
		now = now.Add(time.Minute)
		return current
	})

	log.Append("first")
	log.Append("second")
	log.Append("third")

	entries := log.Entries()
	got := []string{entries[0].Message, entries[1].Message}
	if want := []string{"third", "second"}; !reflect.DeepEqual(got, want) {
		t.Fatalf("messages = %v, want %v", got, want)
	}
	if got := log.Text(); got != "09:02:00  third\n09:01:00  second" {
		t.Fatalf("Text() = %q", got)
	}

	log.Clear()
	if entries := log.Entries(); len(entries) != 0 {
		t.Fatalf("entries after Clear() = %v, want empty", entries)
	}
}
