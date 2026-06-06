package mishon

import (
	"encoding/json"
	"testing"

	"github.com/mtratsiuk/mtratsiuk.github.io/parser"
)

func TestMishon(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		wantJSON string
		wantOk   bool
	}{
		{
			name:     "text",
			input:    "hello",
			wantJSON: `"hello"`,
			wantOk:   true,
		},
		{
			name:     "text with escapes",
			input:    `hello \[world\]`,
			wantJSON: `"hello [world]"`,
			wantOk:   true,
		},
		{
			name:     "array of texts",
			input:    "[foo\nbar]",
			wantJSON: `["foo","bar"]`,
			wantOk:   true,
		},
		{
			name:     "array of arrays of texts",
			input:    "[[a]\n[b]]",
			wantJSON: `[["a"],["b"]]`,
			wantOk:   true,
		},
		{
			name:     "array of arrays and texts",
			input:    "[[a\nb [c [d]]]\n[e]]",
			wantJSON: `[["a","b",["c",["d"]]],["e"]]`,
			wantOk:   true,
		},
		{
			name: "object of texts",
			input: `
			{
				one: a b c
				two: d e f
			}
			`,
			wantJSON: `{"one":"a b c","two":"d e f"}`,
			wantOk:   true,
		},
		{
			name: "object of objects and texts and arrays",
			input: `
			{
				one: a b c
				two: [
					d e f
					g h i
				]
				three: {
					four: j k l
				}
			}
			`,
			wantJSON: `{"one":"a b c","three":{"four":"j k l"},"two":["d e f","g h i"]}`,
			wantOk:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok, _ := parser.Parse(p_mishon, tt.input)

			if ok != tt.wantOk {
				t.Errorf("ok = %v, want %v", ok, tt.wantOk)
				return
			}

			gotJSON, err := json.Marshal(got)
			if err != nil {
				t.Fatalf("json.Marshal: %v", err)
			}

			if string(gotJSON) != tt.wantJSON {
				t.Errorf("got %s, want %s", gotJSON, tt.wantJSON)
			}
		})
	}
}
