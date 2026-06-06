package parser

import (
	"fmt"
	"strconv"
	"unicode"
)

type ParserState struct {
	input string

	line      int
	posInLine int
	pos       int

	errors []error
}

type Parser[T any] func(state *ParserState) (T, bool)

type ParserError struct {
	err             error
	posInLine, line int
}

func (pe ParserError) Error() string {
	return fmt.Sprintf("[line %v; pos %v]: %v", pe.line, pe.posInLine, pe.err)
}

func (ps *ParserState) Advance() {
	if ps.input[ps.pos] == '\n' {
		ps.line += 1
		ps.posInLine = 0
	} else {
		ps.posInLine += 1
	}

	ps.pos += 1
}

func (ps *ParserState) Peek() (byte, bool) {
	if ps.IsEof() {
		return 0, false
	}

	return ps.input[ps.pos], true
}

func (ps *ParserState) MustPeek() byte {
	if r, ok := ps.Peek(); ok {
		return r
	}

	panic("MustPeek: can't peek at eof")
}

func (ps *ParserState) MustPeekBehind() byte {
	if ps.pos == 0 {
		panic("MustPeekBehind: can't peek at pos 0")
	}

	return ps.input[ps.pos-1]
}

func (ps *ParserState) IsEof() bool {
	return ps.pos >= len(ps.input)
}

func Parse[T any](parser Parser[T], input string) (T, bool, ParserState) {
	state := ParserState{
		input:     input,
		line:      0,
		posInLine: 0,
		pos:       0,
		errors:    make([]error, 0),
	}

	r, ok := parser(&state)

	return r, ok, state
}

func ByteCond(cond func(byte) bool) Parser[byte] {
	return func(state *ParserState) (byte, bool) {
		char, ok := state.Peek()
		if !ok {
			return 0, false
		}

		if cond(char) {
			state.Advance()
			return char, true
		}

		return 0, false
	}
}

func Byte(char byte) Parser[byte] {
	return ByteCond(func(b byte) bool { return char == b })
}

func NotByte(char byte) Parser[byte] {
	return ByteCond(func(b byte) bool { return char != b })
}

var Digit = ByteCond(func(char byte) bool { return char >= '0' && char <= '9' })
var Whitespace = ByteCond(func(char byte) bool { return unicode.IsSpace(rune(char)) })
var Newline = Byte('\n')
var NotNewline = NotByte('\n')
var SkipWhitespace = ZeroOrMore(Whitespace)

func TakeUntilByte(char byte) Parser[[]byte] {
	return Many(NotByte(char))
}

func Integer(state *ParserState) (int64, bool) {
	if rs, ok := Many(Digit)(state); ok {
		v, err := strconv.ParseInt(string(rs), 10, 64)
		if err != nil {
			state.errors = append(state.errors, fmt.Errorf("Integer: failed to parse: %v", err))
		}

		return v, true
	}

	return 0, false
}

func TakeBytesUntil[T any](parser Parser[T]) Parser[[]byte] {
	return func(state *ParserState) ([]byte, bool) {
		chars := make([]byte, 0)

		for !state.IsEof() {
			initialPos := state.pos

			if _, ok := parser(state); ok {
				state.pos = initialPos
				break
			} else {
				chars = append(chars, state.MustPeek())
				state.Advance()
			}
		}

		return chars, len(chars) > 0
	}
}

func Many[T any](parser Parser[T]) Parser[[]T] {
	return func(state *ParserState) ([]T, bool) {
		rs := make([]T, 0)

		for !state.IsEof() {
			if r, ok := parser(state); ok {
				rs = append(rs, r)
			} else {
				break
			}
		}

		return rs, len(rs) > 0
	}
}

func Or[T any](parsers ...Parser[T]) Parser[T] {
	return func(state *ParserState) (T, bool) {
		for _, parser := range parsers {
			r, ok := parser(state)

			if ok {
				return r, true
			}
		}

		var zero T
		return zero, false
	}
}

func Map[T, R any](parser Parser[T], mapV func(T) R) Parser[R] {
	return func(state *ParserState) (R, bool) {
		if r, ok := parser(state); ok {
			return mapV(r), true
		} else {
			var zero R
			return zero, ok
		}
	}
}

func MapError[T, R any](parser Parser[T], mapV func(T) R, mapE func() R) Parser[R] {
	return func(state *ParserState) (R, bool) {
		if r, ok := parser(state); ok {
			return mapV(r), true
		} else {
			return mapE(), true
		}
	}
}

func ZeroOrMore[T any](parser Parser[T]) Parser[[]T] {
	return func(state *ParserState) ([]T, bool) {
		rs, _ := Many(parser)(state)
		return rs, true
	}
}

func NegativeLookBehind[T any](parser Parser[T], char byte) Parser[T] {
	return func(state *ParserState) (T, bool) {
		initialPos := state.pos
		lookBehindPassed := initialPos == 0 || state.MustPeekBehind() != char

		if lookBehindPassed {
			return parser(state)
		}

		var zero T
		return zero, false
	}
}

type seqBreakSignal struct{}

func Seq[T any](block SeqBlock[T]) Parser[T] {
	return func(state *ParserState) (rv T, rok bool) {
		initialPos := state.pos

		defer func() {
			if err := recover(); err != nil {
				if _, ok := err.(seqBreakSignal); ok {
					var zero T
					rv = zero
					rok = false
					state.pos = initialPos
				} else {
					panic(err)
				}
			}
		}()

		return block(state), true
	}
}

type SeqBlock[T any] func(state *ParserState) T

func Emit[T any](parser Parser[T], state *ParserState) T {
	if r, ok := parser(state); ok {
		return r
	} else {
		panic(seqBreakSignal{})
	}
}
