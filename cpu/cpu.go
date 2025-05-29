package cpu

// CPU represents the Gameboy CPU (Sharp LR35902)
type CPU struct {
	// Registers, flags, etc. will go here
}

// New returns a new CPU instance
func New() *CPU {
	return &CPU{}
}

// Step executes a single CPU instruction
func (c *CPU) Step() {
	// Instruction execution logic will go here
}
