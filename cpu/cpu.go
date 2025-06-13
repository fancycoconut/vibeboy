package cpu

import "fmt"

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

// Step executes a single CPU instruction (fetch-decode-execute)
func (c *CPU) Step(mem interface{}) {
	m, ok := mem.(interface {
		Read(addr uint16) byte
		Write(addr uint16, value byte)
	})
	if !ok {
		return // Invalid memory interface
	}

	// Fetch
	opcode := m.Read(c.PC)
	fmt.Printf("[CPU] PC=%04X OPCODE=%02X\n", c.PC, opcode)
	c.PC++

	// Decode & Execute (NOP 0x00, JP nn 0xC3, LD BC,nn 0x01, INC B 0x04, DEC B 0x05, LD (BC),A 0x02, LD B,n 0x06, LD A,(BC) 0x0A, INC C 0x0C, DEC C 0x0D, LD C,n 0x0E)
	switch opcode {
	case 0x00: // NOP
		fmt.Println("[CPU] Executed NOP")
	case 0xC3: // JP nn (Jump to 16-bit immediate address)
		low := m.Read(c.PC)
		c.PC++
		high := m.Read(c.PC)
		c.PC++
		addr := uint16(high)<<8 | uint16(low)
		fmt.Printf("[CPU] Executed JP %04X\n", addr)
		c.PC = addr
	case 0x01: // LD BC,nn (Load 16-bit immediate into BC)
		low := m.Read(c.PC)
		c.PC++
		high := m.Read(c.PC)
		c.PC++
		c.B = high
		c.C = low
		fmt.Printf("[CPU] Executed LD BC,%02X%02X\n", high, low)
	case 0x04: // INC B
		c.B++
		fmt.Printf("[CPU] Executed INC B, B=%02X\n", c.B)
	case 0x05: // DEC B
		c.B--
		fmt.Printf("[CPU] Executed DEC B, B=%02X\n", c.B)
	case 0x02: // LD (BC),A
		addr := uint16(c.B)<<8 | uint16(c.C)
		m.Write(addr, c.A)
		fmt.Printf("[CPU] Executed LD (BC),A, (BC)=%04X, A=%02X\n", addr, c.A)
	case 0x06: // LD B,n
		val := m.Read(c.PC)
		c.PC++
		c.B = val
		fmt.Printf("[CPU] Executed LD B,%02X\n", val)
	case 0x0A: // LD A,(BC)
		addr := uint16(c.B)<<8 | uint16(c.C)
		c.A = m.Read(addr)
		fmt.Printf("[CPU] Executed LD A,(BC), (BC)=%04X, A=%02X\n", addr, c.A)
	case 0x0C: // INC C
		c.C++
		fmt.Printf("[CPU] Executed INC C, C=%02X\n", c.C)
	case 0x0D: // DEC C
		c.C--
		fmt.Printf("[CPU] Executed DEC C, C=%02X\n", c.C)
	case 0x0E: // LD C,n
		val := m.Read(c.PC)
		c.PC++
		c.C = val
		fmt.Printf("[CPU] Executed LD C,%02X\n", val)
	default:
		fmt.Printf("[CPU] Unimplemented opcode: %02X\n", opcode)
	}
}
