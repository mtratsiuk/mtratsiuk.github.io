package mishon

import (
	"encoding/json"
	"fmt"
	"strings"

	"github.com/mtratsiuk/mtratsiuk.github.io/parser"
)

type ValueClass int

const (
	ValueClassText ValueClass = iota
	ValueClassArray
	ValueClassObject
)

type Value struct {
	class  ValueClass
	text   string
	array  []Value
	object map[string]Value
}

func (v Value) Text() string {
	v.assertValueClass(ValueClassText)
	return v.text
}

func (v Value) Array() []Value {
	v.assertValueClass(ValueClassArray)
	return v.array
}

func (v Value) Object() map[string]Value {
	v.assertValueClass(ValueClassObject)
	return v.object
}

func (v Value) MarshalJSON() ([]byte, error) {
	switch v.class {
	case ValueClassArray:
		return json.Marshal(v.array)
	case ValueClassObject:
		return json.Marshal(v.object)
	case ValueClassText:
		return json.Marshal(v.text)
	default:
		panic(fmt.Sprintf("unexpected gostach.ValueClass: %#v", v.class))
	}
}

func (v Value) assertValueClass(class ValueClass) {
	if v.class != class {
		panic(fmt.Sprintf("expected class %v when reading payload of value %v", class, v))
	}
}

func NewValueText(text string) Value {
	return Value{class: ValueClassText, text: text}
}

func NewValueArray(array []Value) Value {
	return Value{class: ValueClassArray, array: array}
}

func NewValueObject(object map[string]Value) Value {
	return Value{class: ValueClassObject, object: object}
}

var p_objectOpen = parser.NegativeLookBehind(parser.Byte('{'), '\\')
var p_objectClose = parser.NegativeLookBehind(parser.Byte('}'), '\\')
var p_arrayOpen = parser.NegativeLookBehind(parser.Byte('['), '\\')
var p_arrayClose = parser.NegativeLookBehind(parser.Byte(']'), '\\')
var p_idClose = parser.NegativeLookBehind(parser.Byte(':'), '\\')

var p_bytesUntilSpecial = parser.TakeBytesUntil(
	parser.Or(
		p_objectOpen,
		p_objectClose,
		p_arrayOpen,
		p_arrayClose,
		p_idClose,
		parser.Newline,
	),
)

var p_mishon parser.Parser[Value]

var p_text = parser.Seq(func(state *parser.ParserState) Value {
	parser.Emit(parser.SkipWhitespace, state)
	textBytes := parser.Emit(p_bytesUntilSpecial, state)
	parser.Emit(parser.SkipWhitespace, state)

	text := string(textBytes)
	text = strings.ReplaceAll(text, "\\{", "{")
	text = strings.ReplaceAll(text, "\\}", "}")
	text = strings.ReplaceAll(text, "\\[", "[")
	text = strings.ReplaceAll(text, "\\]", "]")
	text = strings.ReplaceAll(text, "\\:", ":")

	return NewValueText(strings.TrimSpace(text))
})

var p_array = parser.Seq(func(state *parser.ParserState) Value {
	parser.Emit(parser.SkipWhitespace, state)
	parser.Emit(p_arrayOpen, state)
	values := parser.Emit(parser.Many(p_mishon), state)
	parser.Emit(p_arrayClose, state)
	parser.Emit(parser.SkipWhitespace, state)

	return NewValueArray(values)
})

type objectEntry struct {
	key   string
	value Value
}

var p_objectEntry = parser.Seq(func(state *parser.ParserState) objectEntry {
	parser.Emit(parser.SkipWhitespace, state)
	key := parser.Emit(p_bytesUntilSpecial, state)
	parser.Emit(p_idClose, state)
	parser.Emit(parser.SkipWhitespace, state)
	value := parser.Emit(p_mishon, state)
	parser.Emit(parser.SkipWhitespace, state)

	return objectEntry{strings.TrimSpace(string(key)), value}
})

var p_object = parser.Seq(func(state *parser.ParserState) Value {
	parser.Emit(parser.SkipWhitespace, state)
	parser.Emit(p_objectOpen, state)

	entries := make(map[string]Value, 0)

	for _, entry := range parser.Emit(parser.Many(p_objectEntry), state) {
		entries[entry.key] = entry.value
	}

	parser.Emit(p_objectClose, state)
	parser.Emit(parser.SkipWhitespace, state)

	return NewValueObject(entries)
})

func init() {
	p_mishon = parser.Or(
		p_array,
		p_object,
		p_text,
	)
}
