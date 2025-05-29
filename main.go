package main

import (
	"fmt"
	"os"
	"vibeboy/rom"
)

func main() {
	fmt.Println("Vibeboy Gameboy Emulator - WIP")

	if len(os.Args) < 2 {
		fmt.Println("Usage: vibeboy <romfile>")
		return
	}

	romPath := os.Args[1]
	gameROM, err := rom.Load(romPath)
	if err != nil {
		fmt.Printf("Failed to load ROM: %v\n", err)
		return
	}
	fmt.Printf("Loaded ROM: %s (%d bytes)\n", romPath, len(gameROM.Data))
}
