package cpu

import (
	"testing"
	"vibeboy/memory"
)

type dummyMem struct {
	mem *memory.Memory
}

func (d *dummyMem) Read(addr uint16) byte  { return d.mem.Read(addr) }
func (d *dummyMem) Write(addr uint16, v byte) { d.mem.Write(addr, v) }

func TestNOP(t *testing.T) {
	mem := memory.New()
	mem.Write(0x0100, 0x00) // NOP
	cpu := New()
	cpu.Step(mem)
	if cpu.PC != 0x0101 {
		t.Errorf("NOP: expected PC=0x0101, got %04X", cpu.PC)
	}
}

func TestJP(t *testing.T) {
	mem := memory.New()
	mem.Write(0x0100, 0xC3) // JP nn
	mem.Write(0x0101, 0x34)
	mem.Write(0x0102, 0x12)
	cpu := New()
	cpu.Step(mem)
	if cpu.PC != 0x1234 {
		t.Errorf("JP: expected PC=0x1234, got %04X", cpu.PC)
	}
}

func TestLDBCnn(t *testing.T) {
	mem := memory.New()
	mem.Write(0x0100, 0x01) // LD BC,nn
	mem.Write(0x0101, 0xCD)
	mem.Write(0x0102, 0xAB)
	cpu := New()
	cpu.Step(mem)
	if cpu.B != 0xAB || cpu.C != 0xCD {
		t.Errorf("LD BC,nn: expected B=0xAB, C=0xCD, got B=%02X, C=%02X", cpu.B, cpu.C)
	}
}

func TestINCB(t *testing.T) {
	mem := memory.New()
	mem.Write(0x0100, 0x04) // INC B
	cpu := New()
	cpu.B = 0x12
	cpu.Step(mem)
	if cpu.B != 0x13 {
		t.Errorf("INC B: expected B=0x13, got %02X", cpu.B)
	}
}

func TestDECB(t *testing.T) {
	mem := memory.New()
	mem.Write(0x0100, 0x05) // DEC B
	cpu := New()
	cpu.B = 0x12
	cpu.Step(mem)
	if cpu.B != 0x11 {
		t.Errorf("DEC B: expected B=0x11, got %02X", cpu.B)
	}
}

func TestLDBC_A(t *testing.T) {
	mem := memory.New()
	mem.Write(0x0100, 0x02) // LD (BC),A
	cpu := New()
	cpu.B = 0x12
	cpu.C = 0x34
	cpu.A = 0x56
	cpu.Step(mem)
	if mem.Read(0x1234) != 0x56 {
		t.Errorf("LD (BC),A: expected mem[0x1234]=0x56, got %02X", mem.Read(0x1234))
	}
}
