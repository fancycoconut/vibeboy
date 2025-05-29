package rom

import (
	"errors"
	"io/ioutil"
)

// ROM represents a loaded Gameboy ROM
type ROM struct {
	Data []byte
}

// hasValidExtension checks if the file has a .gb or .gbc extension
func hasValidExtension(path string) bool {
	ext := ""
	if len(path) > 3 {
		ext = path[len(path)-3:]
	}
	if len(path) > 4 && path[len(path)-4:] == ".gbc" {
		return true
	}
	return ext == ".gb"
}

// Load loads a ROM from the given file path and checks extension
func Load(path string) (*ROM, error) {
	if !hasValidExtension(path) {
		return nil, errors.New("ROM file must have .gb or .gbc extension")
	}
	data, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return &ROM{Data: data}, nil
}
