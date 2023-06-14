package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"
)

const (
	greenColor   = "\033[1;32m"
	resetColor   = "\033[0m"
	clearConsole = "\033[H\033[2J"
)

var bigDigits = map[string]string{
	"0": `
 ██████╗ 
██╔═████╗
██║██╔██║
████╔╝██║
╚██████╔╝
 ╚═════╝ `,
	"1": `
   ██╗   
 ███║   
 ╚██║   
  ██║   
  ██║   
  ╚═╝   `,
	"2": `
 ██████╗ 
██╔════╝ 
██║  ███╗
██║   ██║
╚██████╔╝
 ╚═════╝ `,
	"3": `
 ██████╗ 
██╔════╝ 
╚█████╗  
 ╚═══██╗ 
 █████╔╝ 
 ╚════╝  `,
	"4": `
██╗  ██╗
██║  ██║
███████║
╚════██║
     ██║
     ╚═╝`,
	"5": `
██████╗ 
██╔═══╝ 
██████╗ 
╚═══██╗
██████╔╝
╚═════╝ `,
	"6": `
 ██████╗ 
██╔═══██╗
██████╔╝
██╔═══╝ 
╚██████╗
 ╚═════╝ `,
	"7": `
███████╗
╚════██║
    ██╔╝
   ██╔╝ 
   ██║  
   ╚═╝  `,
	"8": `
 █████╗ 
██╔══██╗
╚█████╔╝
██╔══██╗
╚█████╔╝
 ╚════╝ `,
	"9": `
 █████╗ 
██╔══██╗
╚██████║
 ╚═══██║
 █████╔╝
 ╚════╝ `,
	":": `
    
  ██╗
  ╚═╝
  ██╗
    
`}

func main() {
	for {
		// Clear the console
		cmd := exec.Command("bash", "-c", "clear")
		cmd.Stdout = os.Stdout
		cmd.Run()

		// Get the current time
		currentTime := time.Now()
		hourString := currentTime.Format("03")
		minuteString := currentTime.Format("04")
		secondString := currentTime.Format("05")
		amPmString := currentTime.Format("PM")
		dateString := currentTime.Format("January 02, 2006")

		// Calculate the center alignment
		terminalWidth, _, err := terminalSize()
		if err != nil {
			fmt.Println("Error getting terminal size:", err)
			return
		}
		padding := strings.Repeat(" ", (terminalWidth-54)/2)

		// Print the time and date
		printBigDigit(padding, bigDigits[hourString[0:1]])
		printBigDigit(padding, bigDigits[hourString[1:2]])
		fmt.Println()
		printBigDigit(padding, strings.ReplaceAll(bigDigits[":"], " ", greenColor+bold(" ")+resetColor))
		fmt.Println()
		printBigDigit(padding, bigDigits[minuteString[0:1]])
		printBigDigit(padding, bigDigits[minuteString[1:2]])
		fmt.Println()
		printBigDigit(padding, strings.ReplaceAll(bigDigits[":"], " ", greenColor+bold(" ")+resetColor))
		fmt.Println()
		printBigDigit(padding, bigDigits[secondString[0:1]])
		printBigDigit(padding, bigDigits[secondString[1:2]])
		fmt.Println()
		printBigDigit(padding, bigDigits[amPmString])
		fmt.Println()
		fmt.Printf("%s%s\n", padding, dateString)

		// Wait for 1 second before updating the time
		time.Sleep(1 * time.Second)
	}
}

// terminalSize returns the width, height, and any error encountered while getting the terminal size
func terminalSize() (int, int, error) {
	cmd := exec.Command("bash", "-c", "stty size")
	cmd.Stdin = os.Stdin
	out, err := cmd.Output()
	if err != nil {
		return 0, 0, err
	}

	result := strings.Split(string(out), " ")
	width, height := 0, 0
	fmt.Sscanf(result[1], "%d", &width)
	fmt.Sscanf(result[0], "%d", &height)
	return width, height, nil
}

// printBigDigit prints the big digit representation with the specified padding
func printBigDigit(padding, digit string) {
	lines := strings.Split(digit, "\n")
	for _, line := range lines {
		fmt.Printf("%s%s\n", padding, line)
	}
}

// bold returns the string formatted with bold font
func bold(s string) string {
	return "\033[1m" + s + "\033[0m"
}
