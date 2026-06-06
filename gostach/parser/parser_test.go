package parser

import (
	"testing"
)

func TestSeqSum(t *testing.T) {
	var Sum = Seq(func(state *ParserState) int64 {
		left := Emit(Integer, state)
		Emit(Whitespace, state)
		right := Emit(Integer, state)
		return left + right
	})

	tests := []struct {
		input  string
		want   int64
		wantOk bool
	}{
		{"1 2", 3, true},
		{"10 20", 30, true},
		{"0 0", 0, true},
		{"123 456", 579, true},
		{"1 abc", 0, false},
		{"1abc", 0, false},
		{"abc 2", 0, false},
		{"", 0, false},
	}

	for _, tt := range tests {
		got, ok, _ := Parse(Sum, tt.input)

		if ok != tt.wantOk {
			t.Errorf("Parse(Sum, %q) ok = %v, want %v", tt.input, ok, tt.wantOk)
			continue
		}

		if ok && got != tt.want {
			t.Errorf("Parse(Sum, %q) = %v, want %v", tt.input, got, tt.want)
		}
	}
}

func TestInteger(t *testing.T) {
	tests := []struct {
		input  string
		want   int64
		wantOk bool
	}{
		{"123", 123, true},
		{"0", 0, true},
		{"9876543210", 9876543210, true},
		{"42abc", 42, true},
		{"", 0, false},
		{"abc", 0, false},
		{"-1", 0, false},
	}

	for _, tt := range tests {
		got, ok, _ := Parse(Integer, tt.input)

		if ok != tt.wantOk {
			t.Errorf("Parse(Integer, %q) ok = %v, want %v", tt.input, ok, tt.wantOk)
			continue
		}

		if ok && got != tt.want {
			t.Errorf("Parse(Integer, %q) = %v, want %v", tt.input, got, tt.want)
		}
	}
}
