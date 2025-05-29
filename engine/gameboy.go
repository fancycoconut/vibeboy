package engine

import (
	"vibeboy/cpu"
	"vibeboy/input"
	"vibeboy/memory"
	"vibeboy/ppu"
	"vibeboy/rom"
)

// Gameboy encapsulates all major emulator components
type Gameboy struct {
	CPU    *cpu.CPU
	Memory *memory.Memory
	PPU    *ppu.PPU
	Input  *input.Input
	ROM    *rom.ROM
}

// NewGameboy creates and connects all emulator components
func NewGameboy(romData *rom.ROM) *Gameboy {
	mem := memory.New()
	mem.LoadROM(romData.Data)
	return &Gameboy{
		CPU:    cpu.New(),
		Memory: mem,
		PPU:    ppu.New(),
		Input:  input.New(),
		ROM:    romData,
	}
}
