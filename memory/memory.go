package memory

// Memory represents the Gameboy memory map
type Memory struct {
	data [0x10000]byte // 64KB address space
}

// New returns a new Memory instance
func New() *Memory {
	return &Memory{}
}

// Read returns the byte at the given address
func (m *Memory) Read(addr uint16) byte {
	return m.data[addr]
}

// Write sets the byte at the given address
func (m *Memory) Write(addr uint16, value byte) {
	m.data[addr] = value
}
