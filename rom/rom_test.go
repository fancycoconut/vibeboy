package rom

import (
	"os"
	"testing"
)

func TestHasValidExtension(t *testing.T) {
	cases := []struct {
		name string
		path string
		valid bool
	}{
		{"GB extension", "game.gb", true},
		{"GBC extension", "game.gbc", true},
		{"Uppercase extension", "game.GB", false},
		{"No extension", "game", false},
		{"Wrong extension", "game.txt", false},
		{"Longer filename", "mygamefile.gbc", true},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := hasValidExtension(c.path); got != c.valid {
				t.Errorf("hasValidExtension(%q) = %v, want %v", c.path, got, c.valid)
			}
		})
	}
}

func TestLoad_InvalidExtension(t *testing.T) {
	_, err := Load("invalid.txt")
	if err == nil {
		t.Error("expected error for invalid extension, got nil")
	}
}

func TestLoad_FileNotFound(t *testing.T) {
	_, err := Load("notfound.gb")
	if err == nil {
		t.Error("expected error for missing file, got nil")
	}
}

func TestLoad_ValidButEmptyFile(t *testing.T) {
	fname := "test.gb"
	os.WriteFile(fname, []byte{}, 0644)
	defer os.Remove(fname)
	_, err := Load(fname)
	if err != nil {
		t.Errorf("expected no error for empty .gb file, got %v", err)
	}
}
