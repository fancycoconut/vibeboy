package rom

import (
	"io/ioutil"
)

// ROM represents a loaded Gameboy ROM
type ROM struct {
	Data []byte
}

// Load loads a ROM from the given file path
func Load(path string) (*ROM, error) {
	data, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return &ROM{Data: data}, nil
}
