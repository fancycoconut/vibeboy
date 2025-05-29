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

// Step runs one emulation cycle (CPU, PPU, etc.)
func (g *Gameboy) Step() {
	g.CPU.Step(g.Memory)
	g.PPU.Step()
	// Input and timers would also be stepped here in a full implementation
}
