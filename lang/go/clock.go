package main

import (
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"
)

func main() {
	clearScreen()
	for {
		printClock()
		time.Sleep(1 * time.Second)
		clearScreen()
	}
}

func clearScreen() {
	cmd := exec.Command("clear") // For Windows, use "cls" instead
	cmd.Stdout = os.Stdout
	cmd.Run()
}

func printClock() {
	now := time.Now()
	timeStr := now.Format("03:04:05 [PM]")
	dateStr := now.Format("2006-01-02")

	terminalWidth, terminalHeight := getTerminalSize()
	clockWidth := len(timeStr)
	clockHeight := 1

	clockStartX := (terminalWidth - clockWidth) / 2
	clockStartY := (terminalHeight - clockHeight) / 2

	fmt.Printf("\033[%d;%dH", clockStartY, clockStartX)
	fmt.Printf("\033[1;32m%s\033[0m\n", timeStr)

	dateStartX := (terminalWidth - len(dateStr)) / 2
	dateStartY := clockStartY + clockHeight + 1

	fmt.Printf("\033[%d;%dH", dateStartY, dateStartX)
	fmt.Printf("\033[1;32m%s\033[0m\n", dateStr)
}


func getTerminalSize() (int, int) {
	cmd := exec.Command("stty", "size")
	cmd.Stdin = os.Stdin
	out, _ := cmd.Output()

	sizeStr := strings.TrimSpace(string(out))
	sizeArr := strings.Split(sizeStr, " ")

	width, _ := strconv.Atoi(sizeArr[1])
	height, _ := strconv.Atoi(sizeArr[0])

	return width, height
}
