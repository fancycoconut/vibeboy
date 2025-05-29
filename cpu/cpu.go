package cpu

// CPU represents the Gameboy CPU (Sharp LR35902)
type CPU struct {
	// 8-bit registers
	A, F byte // Accumulator & Flags
	B, C byte
	D, E byte
	H, L byte

	// 16-bit registers
	SP uint16 // Stack Pointer
	PC uint16 // Program Counter

	// Internal state (add more as needed)
}

// New returns a new CPU instance
func New() *CPU {
	return &CPU{
		// Gameboy starts with PC at 0x0100, SP at 0xFFFE (typical)
		PC: 0x0100,
		SP: 0xFFFE,
	}
}

// Step executes a single CPU instruction (to be implemented)
func (c *CPU) Step(mem interface{}) {
	// Fetch-decode-execute logic will go here
}
