# Actual frontend demonstrations

Generated with Rust 1.85.1 release inspector on Linux, 2026-10-05. These are syntax demonstrations against the Phase 3 candidate. No program is executed. All 14 canonical examples returned exit 0 for both commands. No generic declaration or async example exists in the v0.1 candidate.

## 01-hello-world.cretes

```cretes
import std::io;

fn main() -> Result[i32, io::Error] {
    io::println("Hello, Cretes!")?;
    return Result::Ok(0);
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Import 0..6 "import"
Ident 7..10 "std"
ColonColon 10..12 "::"
Ident 12..14 "io"
Semi 14..15 ";"
Fn 17..19 "fn"
Ident 20..24 "main"
LParen 24..25 "("
RParen 25..26 ")"
Arrow 27..29 "->"
Ident 30..36 "Result"
LBracket 36..37 "["
Ident 37..40 "i32"
Comma 40..41 ","
Ident 42..44 "io"
ColonColon 44..46 "::"
Ident 46..51 "Error"
RBracket 51..52 "]"
LBrace 53..54 "{"
Ident 59..61 "io"
ColonColon 61..63 "::"
Ident 63..70 "println"
LParen 70..71 "("
String 71..87 "\"Hello, Cretes!\""
RParen 87..88 ")"
Question 88..89 "?"
Semi 89..90 ";"
Return 95..101 "return"
Ident 102..108 "Result"
ColonColon 108..110 "::"
Ident 110..112 "Ok"
LParen 112..113 "("
Int 113..114 "0"
RParen 114..115 ")"
Semi 115..116 ";"
RBrace 117..118 "}"
Eof 119..119 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:0..15 Import { path: [Name { span: Span { source: SourceId(0), start: 7, end: 10 } }, Name { span: Span { source: SourceId(0), start: 12, end: 14 } }], alias: None }
1 @0:37..40 NamedType { path: [Name { span: Span { source: SourceId(0), start: 37, end: 40 } }], arguments: [] }
2 @0:42..51 NamedType { path: [Name { span: Span { source: SourceId(0), start: 42, end: 44 } }, Name { span: Span { source: SourceId(0), start: 46, end: 51 } }], arguments: [] }
3 @0:30..52 NamedType { path: [Name { span: Span { source: SourceId(0), start: 30, end: 36 } }], arguments: [NodeId(1), NodeId(2)] }
4 @0:59..70 Path { components: [Name { span: Span { source: SourceId(0), start: 59, end: 61 } }, Name { span: Span { source: SourceId(0), start: 63, end: 70 } }] }
5 @0:71..87 Literal { kind: String }
6 @0:59..88 Call { callee: NodeId(4), arguments: [NodeId(5)] }
7 @0:59..89 Propagate { value: NodeId(6) }
8 @0:59..90 ExpressionStatement { value: NodeId(7) }
9 @0:102..112 Path { components: [Name { span: Span { source: SourceId(0), start: 102, end: 108 } }, Name { span: Span { source: SourceId(0), start: 110, end: 112 } }] }
10 @0:113..114 Literal { kind: Int }
11 @0:102..115 Call { callee: NodeId(9), arguments: [NodeId(10)] }
12 @0:95..116 Return { value: Some(NodeId(11)) }
13 @0:53..118 Block { statements: [NodeId(8), NodeId(12)] }
14 @0:17..118 Function { name: Name { span: Span { source: SourceId(0), start: 20, end: 24 } }, parameters: [], result: NodeId(3), from: None, body: NodeId(13) }
15 @0:17..118 Item { public: false, declaration: NodeId(14) }
16 @0:0..119 Program { module: None, imports: [NodeId(0)], items: [NodeId(15)] }
```

</details>

## 02-variables.cretes

```cretes
const LIMIT: i64 = 10;

fn main() -> i32 {
    let start: i64 = 2;
    var count = start;
    count = count + LIMIT;
    discard(count, "Demonstrate mutable arithmetic without output");
    return 0;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Const 0..5 "const"
Ident 6..11 "LIMIT"
Colon 11..12 ":"
Ident 13..16 "i64"
Eq 17..18 "="
Int 19..21 "10"
Semi 21..22 ";"
Fn 24..26 "fn"
Ident 27..31 "main"
LParen 31..32 "("
RParen 32..33 ")"
Arrow 34..36 "->"
Ident 37..40 "i32"
LBrace 41..42 "{"
Let 47..50 "let"
Ident 51..56 "start"
Colon 56..57 ":"
Ident 58..61 "i64"
Eq 62..63 "="
Int 64..65 "2"
Semi 65..66 ";"
Var 71..74 "var"
Ident 75..80 "count"
Eq 81..82 "="
Ident 83..88 "start"
Semi 88..89 ";"
Ident 94..99 "count"
Eq 100..101 "="
Ident 102..107 "count"
Plus 108..109 "+"
Ident 110..115 "LIMIT"
Semi 115..116 ";"
Ident 121..128 "discard"
LParen 128..129 "("
Ident 129..134 "count"
Comma 134..135 ","
String 136..183 "\"Demonstrate mutable arithmetic without output\""
RParen 183..184 ")"
Semi 184..185 ";"
Return 190..196 "return"
Int 197..198 "0"
Semi 198..199 ";"
RBrace 200..201 "}"
Eof 202..202 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:13..16 NamedType { path: [Name { span: Span { source: SourceId(0), start: 13, end: 16 } }], arguments: [] }
1 @0:19..21 Literal { kind: Int }
2 @0:0..22 Constant { name: Name { span: Span { source: SourceId(0), start: 6, end: 11 } }, ty: NodeId(0), value: NodeId(1) }
3 @0:0..22 Item { public: false, declaration: NodeId(2) }
4 @0:37..40 NamedType { path: [Name { span: Span { source: SourceId(0), start: 37, end: 40 } }], arguments: [] }
5 @0:58..61 NamedType { path: [Name { span: Span { source: SourceId(0), start: 58, end: 61 } }], arguments: [] }
6 @0:64..65 Literal { kind: Int }
7 @0:47..66 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 51, end: 56 } }, ty: Some(NodeId(5)), value: NodeId(6) }
8 @0:83..88 Path { components: [Name { span: Span { source: SourceId(0), start: 83, end: 88 } }] }
9 @0:71..89 Binding { mutable: true, name: Name { span: Span { source: SourceId(0), start: 75, end: 80 } }, ty: None, value: NodeId(8) }
10 @0:94..99 Path { components: [Name { span: Span { source: SourceId(0), start: 94, end: 99 } }] }
11 @0:102..107 Path { components: [Name { span: Span { source: SourceId(0), start: 102, end: 107 } }] }
12 @0:110..115 Path { components: [Name { span: Span { source: SourceId(0), start: 110, end: 115 } }] }
13 @0:102..115 Binary { operator: Plus, left: NodeId(11), right: NodeId(12) }
14 @0:94..116 Assignment { place: NodeId(10), value: NodeId(13) }
15 @0:121..128 Path { components: [Name { span: Span { source: SourceId(0), start: 121, end: 128 } }] }
16 @0:129..134 Path { components: [Name { span: Span { source: SourceId(0), start: 129, end: 134 } }] }
17 @0:136..183 Literal { kind: String }
18 @0:121..184 Call { callee: NodeId(15), arguments: [NodeId(16), NodeId(17)] }
19 @0:121..185 ExpressionStatement { value: NodeId(18) }
20 @0:197..198 Literal { kind: Int }
21 @0:190..199 Return { value: Some(NodeId(20)) }
22 @0:41..201 Block { statements: [NodeId(7), NodeId(9), NodeId(14), NodeId(19), NodeId(21)] }
23 @0:24..201 Function { name: Name { span: Span { source: SourceId(0), start: 27, end: 31 } }, parameters: [], result: NodeId(4), from: None, body: NodeId(22) }
24 @0:24..201 Item { public: false, declaration: NodeId(23) }
25 @0:0..202 Program { module: None, imports: [], items: [NodeId(3), NodeId(24)] }
```

</details>

## 03-functions.cretes

```cretes
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

fn main() -> i32 {
    let answer = add(20, 22);
    discard(answer, "Teaching example");
    return 0;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Fn 0..2 "fn"
Ident 3..6 "add"
LParen 6..7 "("
Ident 7..8 "a"
Colon 8..9 ":"
Ident 10..13 "i64"
Comma 13..14 ","
Ident 15..16 "b"
Colon 16..17 ":"
Ident 18..21 "i64"
RParen 21..22 ")"
Arrow 23..25 "->"
Ident 26..29 "i64"
LBrace 30..31 "{"
Return 36..42 "return"
Ident 43..44 "a"
Plus 45..46 "+"
Ident 47..48 "b"
Semi 48..49 ";"
RBrace 50..51 "}"
Fn 53..55 "fn"
Ident 56..60 "main"
LParen 60..61 "("
RParen 61..62 ")"
Arrow 63..65 "->"
Ident 66..69 "i32"
LBrace 70..71 "{"
Let 76..79 "let"
Ident 80..86 "answer"
Eq 87..88 "="
Ident 89..92 "add"
LParen 92..93 "("
Int 93..95 "20"
Comma 95..96 ","
Int 97..99 "22"
RParen 99..100 ")"
Semi 100..101 ";"
Ident 106..113 "discard"
LParen 113..114 "("
Ident 114..120 "answer"
Comma 120..121 ","
String 122..140 "\"Teaching example\""
RParen 140..141 ")"
Semi 141..142 ";"
Return 147..153 "return"
Int 154..155 "0"
Semi 155..156 ";"
RBrace 157..158 "}"
Eof 159..159 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:10..13 NamedType { path: [Name { span: Span { source: SourceId(0), start: 10, end: 13 } }], arguments: [] }
1 @0:7..13 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 7, end: 8 } }, ty: NodeId(0) }
2 @0:18..21 NamedType { path: [Name { span: Span { source: SourceId(0), start: 18, end: 21 } }], arguments: [] }
3 @0:15..21 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 15, end: 16 } }, ty: NodeId(2) }
4 @0:26..29 NamedType { path: [Name { span: Span { source: SourceId(0), start: 26, end: 29 } }], arguments: [] }
5 @0:43..44 Path { components: [Name { span: Span { source: SourceId(0), start: 43, end: 44 } }] }
6 @0:47..48 Path { components: [Name { span: Span { source: SourceId(0), start: 47, end: 48 } }] }
7 @0:43..48 Binary { operator: Plus, left: NodeId(5), right: NodeId(6) }
8 @0:36..49 Return { value: Some(NodeId(7)) }
9 @0:30..51 Block { statements: [NodeId(8)] }
10 @0:0..51 Function { name: Name { span: Span { source: SourceId(0), start: 3, end: 6 } }, parameters: [NodeId(1), NodeId(3)], result: NodeId(4), from: None, body: NodeId(9) }
11 @0:0..51 Item { public: false, declaration: NodeId(10) }
12 @0:66..69 NamedType { path: [Name { span: Span { source: SourceId(0), start: 66, end: 69 } }], arguments: [] }
13 @0:89..92 Path { components: [Name { span: Span { source: SourceId(0), start: 89, end: 92 } }] }
14 @0:93..95 Literal { kind: Int }
15 @0:97..99 Literal { kind: Int }
16 @0:89..100 Call { callee: NodeId(13), arguments: [NodeId(14), NodeId(15)] }
17 @0:76..101 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 80, end: 86 } }, ty: None, value: NodeId(16) }
18 @0:106..113 Path { components: [Name { span: Span { source: SourceId(0), start: 106, end: 113 } }] }
19 @0:114..120 Path { components: [Name { span: Span { source: SourceId(0), start: 114, end: 120 } }] }
20 @0:122..140 Literal { kind: String }
21 @0:106..141 Call { callee: NodeId(18), arguments: [NodeId(19), NodeId(20)] }
22 @0:106..142 ExpressionStatement { value: NodeId(21) }
23 @0:154..155 Literal { kind: Int }
24 @0:147..156 Return { value: Some(NodeId(23)) }
25 @0:70..158 Block { statements: [NodeId(17), NodeId(22), NodeId(24)] }
26 @0:53..158 Function { name: Name { span: Span { source: SourceId(0), start: 56, end: 60 } }, parameters: [], result: NodeId(12), from: None, body: NodeId(25) }
27 @0:53..158 Item { public: false, declaration: NodeId(26) }
28 @0:0..159 Program { module: None, imports: [], items: [NodeId(11), NodeId(27)] }
```

</details>

## 04-conditionals.cretes

```cretes
fn sign(value: i64) -> i64 {
    if value < 0 {
        return -1;
    } else if value == 0 {
        return 0;
    } else {
        return 1;
    }
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Fn 0..2 "fn"
Ident 3..7 "sign"
LParen 7..8 "("
Ident 8..13 "value"
Colon 13..14 ":"
Ident 15..18 "i64"
RParen 18..19 ")"
Arrow 20..22 "->"
Ident 23..26 "i64"
LBrace 27..28 "{"
If 33..35 "if"
Ident 36..41 "value"
Lt 42..43 "<"
Int 44..45 "0"
LBrace 46..47 "{"
Return 56..62 "return"
Minus 63..64 "-"
Int 64..65 "1"
Semi 65..66 ";"
RBrace 71..72 "}"
Else 73..77 "else"
If 78..80 "if"
Ident 81..86 "value"
EqEq 87..89 "=="
Int 90..91 "0"
LBrace 92..93 "{"
Return 102..108 "return"
Int 109..110 "0"
Semi 110..111 ";"
RBrace 116..117 "}"
Else 118..122 "else"
LBrace 123..124 "{"
Return 133..139 "return"
Int 140..141 "1"
Semi 141..142 ";"
RBrace 147..148 "}"
RBrace 149..150 "}"
Eof 151..151 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:15..18 NamedType { path: [Name { span: Span { source: SourceId(0), start: 15, end: 18 } }], arguments: [] }
1 @0:8..18 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 8, end: 13 } }, ty: NodeId(0) }
2 @0:23..26 NamedType { path: [Name { span: Span { source: SourceId(0), start: 23, end: 26 } }], arguments: [] }
3 @0:36..41 Path { components: [Name { span: Span { source: SourceId(0), start: 36, end: 41 } }] }
4 @0:44..45 Literal { kind: Int }
5 @0:36..45 Binary { operator: Lt, left: NodeId(3), right: NodeId(4) }
6 @0:64..65 Literal { kind: Int }
7 @0:63..65 Unary { operator: Minus, mutable: false, operand: NodeId(6) }
8 @0:56..66 Return { value: Some(NodeId(7)) }
9 @0:46..72 Block { statements: [NodeId(8)] }
10 @0:81..86 Path { components: [Name { span: Span { source: SourceId(0), start: 81, end: 86 } }] }
11 @0:90..91 Literal { kind: Int }
12 @0:81..91 Binary { operator: EqEq, left: NodeId(10), right: NodeId(11) }
13 @0:109..110 Literal { kind: Int }
14 @0:102..111 Return { value: Some(NodeId(13)) }
15 @0:92..117 Block { statements: [NodeId(14)] }
16 @0:140..141 Literal { kind: Int }
17 @0:133..142 Return { value: Some(NodeId(16)) }
18 @0:123..148 Block { statements: [NodeId(17)] }
19 @0:78..148 If { condition: NodeId(12), then_block: NodeId(15), otherwise: Some(NodeId(18)) }
20 @0:33..148 If { condition: NodeId(5), then_block: NodeId(9), otherwise: Some(NodeId(19)) }
21 @0:27..150 Block { statements: [NodeId(20)] }
22 @0:0..150 Function { name: Name { span: Span { source: SourceId(0), start: 3, end: 7 } }, parameters: [NodeId(1)], result: NodeId(2), from: None, body: NodeId(21) }
23 @0:0..150 Item { public: false, declaration: NodeId(22) }
24 @0:0..151 Program { module: None, imports: [], items: [NodeId(23)] }
```

</details>

## 05-loops.cretes

```cretes
fn total() -> i64 {
    var sum: i64 = 0;
    for value in [1, 2, 3] {
        sum = sum + value;
    }
    var index: i64 = 0;
    while index < 3 {
        index = index + 1;
        if index == 2 {
            continue;
        }
    }
    loop {
        break;
    }
    return sum;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Fn 0..2 "fn"
Ident 3..8 "total"
LParen 8..9 "("
RParen 9..10 ")"
Arrow 11..13 "->"
Ident 14..17 "i64"
LBrace 18..19 "{"
Var 24..27 "var"
Ident 28..31 "sum"
Colon 31..32 ":"
Ident 33..36 "i64"
Eq 37..38 "="
Int 39..40 "0"
Semi 40..41 ";"
For 46..49 "for"
Ident 50..55 "value"
In 56..58 "in"
LBracket 59..60 "["
Int 60..61 "1"
Comma 61..62 ","
Int 63..64 "2"
Comma 64..65 ","
Int 66..67 "3"
RBracket 67..68 "]"
LBrace 69..70 "{"
Ident 79..82 "sum"
Eq 83..84 "="
Ident 85..88 "sum"
Plus 89..90 "+"
Ident 91..96 "value"
Semi 96..97 ";"
RBrace 102..103 "}"
Var 108..111 "var"
Ident 112..117 "index"
Colon 117..118 ":"
Ident 119..122 "i64"
Eq 123..124 "="
Int 125..126 "0"
Semi 126..127 ";"
While 132..137 "while"
Ident 138..143 "index"
Lt 144..145 "<"
Int 146..147 "3"
LBrace 148..149 "{"
Ident 158..163 "index"
Eq 164..165 "="
Ident 166..171 "index"
Plus 172..173 "+"
Int 174..175 "1"
Semi 175..176 ";"
If 185..187 "if"
Ident 188..193 "index"
EqEq 194..196 "=="
Int 197..198 "2"
LBrace 199..200 "{"
Continue 213..221 "continue"
Semi 221..222 ";"
RBrace 231..232 "}"
RBrace 237..238 "}"
Loop 243..247 "loop"
LBrace 248..249 "{"
Break 258..263 "break"
Semi 263..264 ";"
RBrace 269..270 "}"
Return 275..281 "return"
Ident 282..285 "sum"
Semi 285..286 ";"
RBrace 287..288 "}"
Eof 289..289 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:14..17 NamedType { path: [Name { span: Span { source: SourceId(0), start: 14, end: 17 } }], arguments: [] }
1 @0:33..36 NamedType { path: [Name { span: Span { source: SourceId(0), start: 33, end: 36 } }], arguments: [] }
2 @0:39..40 Literal { kind: Int }
3 @0:24..41 Binding { mutable: true, name: Name { span: Span { source: SourceId(0), start: 28, end: 31 } }, ty: Some(NodeId(1)), value: NodeId(2) }
4 @0:60..61 Literal { kind: Int }
5 @0:63..64 Literal { kind: Int }
6 @0:66..67 Literal { kind: Int }
7 @0:59..68 Sequence { elements: [NodeId(4), NodeId(5), NodeId(6)] }
8 @0:79..82 Path { components: [Name { span: Span { source: SourceId(0), start: 79, end: 82 } }] }
9 @0:85..88 Path { components: [Name { span: Span { source: SourceId(0), start: 85, end: 88 } }] }
10 @0:91..96 Path { components: [Name { span: Span { source: SourceId(0), start: 91, end: 96 } }] }
11 @0:85..96 Binary { operator: Plus, left: NodeId(9), right: NodeId(10) }
12 @0:79..97 Assignment { place: NodeId(8), value: NodeId(11) }
13 @0:69..103 Block { statements: [NodeId(12)] }
14 @0:46..103 For { name: Name { span: Span { source: SourceId(0), start: 50, end: 55 } }, iterable: NodeId(7), body: NodeId(13) }
15 @0:119..122 NamedType { path: [Name { span: Span { source: SourceId(0), start: 119, end: 122 } }], arguments: [] }
16 @0:125..126 Literal { kind: Int }
17 @0:108..127 Binding { mutable: true, name: Name { span: Span { source: SourceId(0), start: 112, end: 117 } }, ty: Some(NodeId(15)), value: NodeId(16) }
18 @0:138..143 Path { components: [Name { span: Span { source: SourceId(0), start: 138, end: 143 } }] }
19 @0:146..147 Literal { kind: Int }
20 @0:138..147 Binary { operator: Lt, left: NodeId(18), right: NodeId(19) }
21 @0:158..163 Path { components: [Name { span: Span { source: SourceId(0), start: 158, end: 163 } }] }
22 @0:166..171 Path { components: [Name { span: Span { source: SourceId(0), start: 166, end: 171 } }] }
23 @0:174..175 Literal { kind: Int }
24 @0:166..175 Binary { operator: Plus, left: NodeId(22), right: NodeId(23) }
25 @0:158..176 Assignment { place: NodeId(21), value: NodeId(24) }
26 @0:188..193 Path { components: [Name { span: Span { source: SourceId(0), start: 188, end: 193 } }] }
27 @0:197..198 Literal { kind: Int }
28 @0:188..198 Binary { operator: EqEq, left: NodeId(26), right: NodeId(27) }
29 @0:213..222 Continue
30 @0:199..232 Block { statements: [NodeId(29)] }
31 @0:185..232 If { condition: NodeId(28), then_block: NodeId(30), otherwise: None }
32 @0:148..238 Block { statements: [NodeId(25), NodeId(31)] }
33 @0:132..238 While { condition: NodeId(20), body: NodeId(32) }
34 @0:258..264 Break
35 @0:248..270 Block { statements: [NodeId(34)] }
36 @0:243..270 Loop { body: NodeId(35) }
37 @0:282..285 Path { components: [Name { span: Span { source: SourceId(0), start: 282, end: 285 } }] }
38 @0:275..286 Return { value: Some(NodeId(37)) }
39 @0:18..288 Block { statements: [NodeId(3), NodeId(14), NodeId(17), NodeId(33), NodeId(36), NodeId(38)] }
40 @0:0..288 Function { name: Name { span: Span { source: SourceId(0), start: 3, end: 8 } }, parameters: [], result: NodeId(0), from: None, body: NodeId(39) }
41 @0:0..288 Item { public: false, declaration: NodeId(40) }
42 @0:0..289 Program { module: None, imports: [], items: [NodeId(41)] }
```

</details>

## 06-collections.cretes

```cretes
import std::seq;
import std::map;
import std::set;

fn main() -> i32 {
    var values: Seq[i64] = [2, 4, 6];
    seq::push(&mut values, 8);
    let lookup: Map[text, i64] = map::empty();
    let unique: Set[i64] = set::empty();
    discard(lookup, "Empty map construction example");
    discard(unique, "Empty set construction example");
    for value in &values {
        discard(*value, "Read each element through a shared view");
    }
    return 0;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Import 0..6 "import"
Ident 7..10 "std"
ColonColon 10..12 "::"
Ident 12..15 "seq"
Semi 15..16 ";"
Import 17..23 "import"
Ident 24..27 "std"
ColonColon 27..29 "::"
Ident 29..32 "map"
Semi 32..33 ";"
Import 34..40 "import"
Ident 41..44 "std"
ColonColon 44..46 "::"
Ident 46..49 "set"
Semi 49..50 ";"
Fn 52..54 "fn"
Ident 55..59 "main"
LParen 59..60 "("
RParen 60..61 ")"
Arrow 62..64 "->"
Ident 65..68 "i32"
LBrace 69..70 "{"
Var 75..78 "var"
Ident 79..85 "values"
Colon 85..86 ":"
Ident 87..90 "Seq"
LBracket 90..91 "["
Ident 91..94 "i64"
RBracket 94..95 "]"
Eq 96..97 "="
LBracket 98..99 "["
Int 99..100 "2"
Comma 100..101 ","
Int 102..103 "4"
Comma 103..104 ","
Int 105..106 "6"
RBracket 106..107 "]"
Semi 107..108 ";"
Ident 113..116 "seq"
ColonColon 116..118 "::"
Ident 118..122 "push"
LParen 122..123 "("
Amp 123..124 "&"
Mut 124..127 "mut"
Ident 128..134 "values"
Comma 134..135 ","
Int 136..137 "8"
RParen 137..138 ")"
Semi 138..139 ";"
Let 144..147 "let"
Ident 148..154 "lookup"
Colon 154..155 ":"
Ident 156..159 "Map"
LBracket 159..160 "["
Ident 160..164 "text"
Comma 164..165 ","
Ident 166..169 "i64"
RBracket 169..170 "]"
Eq 171..172 "="
Ident 173..176 "map"
ColonColon 176..178 "::"
Ident 178..183 "empty"
LParen 183..184 "("
RParen 184..185 ")"
Semi 185..186 ";"
Let 191..194 "let"
Ident 195..201 "unique"
Colon 201..202 ":"
Ident 203..206 "Set"
LBracket 206..207 "["
Ident 207..210 "i64"
RBracket 210..211 "]"
Eq 212..213 "="
Ident 214..217 "set"
ColonColon 217..219 "::"
Ident 219..224 "empty"
LParen 224..225 "("
RParen 225..226 ")"
Semi 226..227 ";"
Ident 232..239 "discard"
LParen 239..240 "("
Ident 240..246 "lookup"
Comma 246..247 ","
String 248..280 "\"Empty map construction example\""
RParen 280..281 ")"
Semi 281..282 ";"
Ident 287..294 "discard"
LParen 294..295 "("
Ident 295..301 "unique"
Comma 301..302 ","
String 303..335 "\"Empty set construction example\""
RParen 335..336 ")"
Semi 336..337 ";"
For 342..345 "for"
Ident 346..351 "value"
In 352..354 "in"
Amp 355..356 "&"
Ident 356..362 "values"
LBrace 363..364 "{"
Ident 373..380 "discard"
LParen 380..381 "("
Star 381..382 "*"
Ident 382..387 "value"
Comma 387..388 ","
String 389..430 "\"Read each element through a shared view\""
RParen 430..431 ")"
Semi 431..432 ";"
RBrace 437..438 "}"
Return 443..449 "return"
Int 450..451 "0"
Semi 451..452 ";"
RBrace 453..454 "}"
Eof 455..455 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:0..16 Import { path: [Name { span: Span { source: SourceId(0), start: 7, end: 10 } }, Name { span: Span { source: SourceId(0), start: 12, end: 15 } }], alias: None }
1 @0:17..33 Import { path: [Name { span: Span { source: SourceId(0), start: 24, end: 27 } }, Name { span: Span { source: SourceId(0), start: 29, end: 32 } }], alias: None }
2 @0:34..50 Import { path: [Name { span: Span { source: SourceId(0), start: 41, end: 44 } }, Name { span: Span { source: SourceId(0), start: 46, end: 49 } }], alias: None }
3 @0:65..68 NamedType { path: [Name { span: Span { source: SourceId(0), start: 65, end: 68 } }], arguments: [] }
4 @0:91..94 NamedType { path: [Name { span: Span { source: SourceId(0), start: 91, end: 94 } }], arguments: [] }
5 @0:87..95 NamedType { path: [Name { span: Span { source: SourceId(0), start: 87, end: 90 } }], arguments: [NodeId(4)] }
6 @0:99..100 Literal { kind: Int }
7 @0:102..103 Literal { kind: Int }
8 @0:105..106 Literal { kind: Int }
9 @0:98..107 Sequence { elements: [NodeId(6), NodeId(7), NodeId(8)] }
10 @0:75..108 Binding { mutable: true, name: Name { span: Span { source: SourceId(0), start: 79, end: 85 } }, ty: Some(NodeId(5)), value: NodeId(9) }
11 @0:113..122 Path { components: [Name { span: Span { source: SourceId(0), start: 113, end: 116 } }, Name { span: Span { source: SourceId(0), start: 118, end: 122 } }] }
12 @0:128..134 Path { components: [Name { span: Span { source: SourceId(0), start: 128, end: 134 } }] }
13 @0:123..134 Unary { operator: Amp, mutable: true, operand: NodeId(12) }
14 @0:136..137 Literal { kind: Int }
15 @0:113..138 Call { callee: NodeId(11), arguments: [NodeId(13), NodeId(14)] }
16 @0:113..139 ExpressionStatement { value: NodeId(15) }
17 @0:160..164 NamedType { path: [Name { span: Span { source: SourceId(0), start: 160, end: 164 } }], arguments: [] }
18 @0:166..169 NamedType { path: [Name { span: Span { source: SourceId(0), start: 166, end: 169 } }], arguments: [] }
19 @0:156..170 NamedType { path: [Name { span: Span { source: SourceId(0), start: 156, end: 159 } }], arguments: [NodeId(17), NodeId(18)] }
20 @0:173..183 Path { components: [Name { span: Span { source: SourceId(0), start: 173, end: 176 } }, Name { span: Span { source: SourceId(0), start: 178, end: 183 } }] }
21 @0:173..185 Call { callee: NodeId(20), arguments: [] }
22 @0:144..186 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 148, end: 154 } }, ty: Some(NodeId(19)), value: NodeId(21) }
23 @0:207..210 NamedType { path: [Name { span: Span { source: SourceId(0), start: 207, end: 210 } }], arguments: [] }
24 @0:203..211 NamedType { path: [Name { span: Span { source: SourceId(0), start: 203, end: 206 } }], arguments: [NodeId(23)] }
25 @0:214..224 Path { components: [Name { span: Span { source: SourceId(0), start: 214, end: 217 } }, Name { span: Span { source: SourceId(0), start: 219, end: 224 } }] }
26 @0:214..226 Call { callee: NodeId(25), arguments: [] }
27 @0:191..227 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 195, end: 201 } }, ty: Some(NodeId(24)), value: NodeId(26) }
28 @0:232..239 Path { components: [Name { span: Span { source: SourceId(0), start: 232, end: 239 } }] }
29 @0:240..246 Path { components: [Name { span: Span { source: SourceId(0), start: 240, end: 246 } }] }
30 @0:248..280 Literal { kind: String }
31 @0:232..281 Call { callee: NodeId(28), arguments: [NodeId(29), NodeId(30)] }
32 @0:232..282 ExpressionStatement { value: NodeId(31) }
33 @0:287..294 Path { components: [Name { span: Span { source: SourceId(0), start: 287, end: 294 } }] }
34 @0:295..301 Path { components: [Name { span: Span { source: SourceId(0), start: 295, end: 301 } }] }
35 @0:303..335 Literal { kind: String }
36 @0:287..336 Call { callee: NodeId(33), arguments: [NodeId(34), NodeId(35)] }
37 @0:287..337 ExpressionStatement { value: NodeId(36) }
38 @0:356..362 Path { components: [Name { span: Span { source: SourceId(0), start: 356, end: 362 } }] }
39 @0:355..362 Unary { operator: Amp, mutable: false, operand: NodeId(38) }
40 @0:373..380 Path { components: [Name { span: Span { source: SourceId(0), start: 373, end: 380 } }] }
41 @0:382..387 Path { components: [Name { span: Span { source: SourceId(0), start: 382, end: 387 } }] }
42 @0:381..387 Unary { operator: Star, mutable: false, operand: NodeId(41) }
43 @0:389..430 Literal { kind: String }
44 @0:373..431 Call { callee: NodeId(40), arguments: [NodeId(42), NodeId(43)] }
45 @0:373..432 ExpressionStatement { value: NodeId(44) }
46 @0:363..438 Block { statements: [NodeId(45)] }
47 @0:342..438 For { name: Name { span: Span { source: SourceId(0), start: 346, end: 351 } }, iterable: NodeId(39), body: NodeId(46) }
48 @0:450..451 Literal { kind: Int }
49 @0:443..452 Return { value: Some(NodeId(48)) }
50 @0:69..454 Block { statements: [NodeId(10), NodeId(16), NodeId(22), NodeId(27), NodeId(32), NodeId(37), NodeId(47), NodeId(49)] }
51 @0:52..454 Function { name: Name { span: Span { source: SourceId(0), start: 55, end: 59 } }, parameters: [], result: NodeId(3), from: None, body: NodeId(50) }
52 @0:52..454 Item { public: false, declaration: NodeId(51) }
53 @0:0..455 Program { module: None, imports: [NodeId(0), NodeId(1), NodeId(2)], items: [NodeId(52)] }
```

</details>

## 07-user-types.cretes

```cretes
struct Point {
    pub x: i64,
    pub y: i64,
}

enum State {
    Idle,
    At(Point),
}

type Count = i64;

fn main() -> i32 {
    let point = new Point { x: 3, y: 4 };
    let state = State::At(point);
    match state {
        State::Idle() => { return 0; }
        State::At(p) => {
            discard(p.x + p.y, "Inspect record payload");
            return 0;
        }
    }
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Struct 0..6 "struct"
Ident 7..12 "Point"
LBrace 13..14 "{"
Pub 19..22 "pub"
Ident 23..24 "x"
Colon 24..25 ":"
Ident 26..29 "i64"
Comma 29..30 ","
Pub 35..38 "pub"
Ident 39..40 "y"
Colon 40..41 ":"
Ident 42..45 "i64"
Comma 45..46 ","
RBrace 47..48 "}"
Enum 50..54 "enum"
Ident 55..60 "State"
LBrace 61..62 "{"
Ident 67..71 "Idle"
Comma 71..72 ","
Ident 77..79 "At"
LParen 79..80 "("
Ident 80..85 "Point"
RParen 85..86 ")"
Comma 86..87 ","
RBrace 88..89 "}"
Type 91..95 "type"
Ident 96..101 "Count"
Eq 102..103 "="
Ident 104..107 "i64"
Semi 107..108 ";"
Fn 110..112 "fn"
Ident 113..117 "main"
LParen 117..118 "("
RParen 118..119 ")"
Arrow 120..122 "->"
Ident 123..126 "i32"
LBrace 127..128 "{"
Let 133..136 "let"
Ident 137..142 "point"
Eq 143..144 "="
New 145..148 "new"
Ident 149..154 "Point"
LBrace 155..156 "{"
Ident 157..158 "x"
Colon 158..159 ":"
Int 160..161 "3"
Comma 161..162 ","
Ident 163..164 "y"
Colon 164..165 ":"
Int 166..167 "4"
RBrace 168..169 "}"
Semi 169..170 ";"
Let 175..178 "let"
Ident 179..184 "state"
Eq 185..186 "="
Ident 187..192 "State"
ColonColon 192..194 "::"
Ident 194..196 "At"
LParen 196..197 "("
Ident 197..202 "point"
RParen 202..203 ")"
Semi 203..204 ";"
Match 209..214 "match"
Ident 215..220 "state"
LBrace 221..222 "{"
Ident 231..236 "State"
ColonColon 236..238 "::"
Ident 238..242 "Idle"
LParen 242..243 "("
RParen 243..244 ")"
FatArrow 245..247 "=>"
LBrace 248..249 "{"
Return 250..256 "return"
Int 257..258 "0"
Semi 258..259 ";"
RBrace 260..261 "}"
Ident 270..275 "State"
ColonColon 275..277 "::"
Ident 277..279 "At"
LParen 279..280 "("
Ident 280..281 "p"
RParen 281..282 ")"
FatArrow 283..285 "=>"
LBrace 286..287 "{"
Ident 300..307 "discard"
LParen 307..308 "("
Ident 308..309 "p"
Dot 309..310 "."
Ident 310..311 "x"
Plus 312..313 "+"
Ident 314..315 "p"
Dot 315..316 "."
Ident 316..317 "y"
Comma 317..318 ","
String 319..343 "\"Inspect record payload\""
RParen 343..344 ")"
Semi 344..345 ";"
Return 358..364 "return"
Int 365..366 "0"
Semi 366..367 ";"
RBrace 376..377 "}"
RBrace 382..383 "}"
RBrace 384..385 "}"
Eof 386..386 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:26..29 NamedType { path: [Name { span: Span { source: SourceId(0), start: 26, end: 29 } }], arguments: [] }
1 @0:19..30 Field { public: true, name: Name { span: Span { source: SourceId(0), start: 23, end: 24 } }, ty: NodeId(0) }
2 @0:42..45 NamedType { path: [Name { span: Span { source: SourceId(0), start: 42, end: 45 } }], arguments: [] }
3 @0:35..46 Field { public: true, name: Name { span: Span { source: SourceId(0), start: 39, end: 40 } }, ty: NodeId(2) }
4 @0:0..48 Record { name: Name { span: Span { source: SourceId(0), start: 7, end: 12 } }, fields: [NodeId(1), NodeId(3)] }
5 @0:0..48 Item { public: false, declaration: NodeId(4) }
6 @0:67..71 Variant { name: Name { span: Span { source: SourceId(0), start: 67, end: 71 } }, payload: [] }
7 @0:80..85 NamedType { path: [Name { span: Span { source: SourceId(0), start: 80, end: 85 } }], arguments: [] }
8 @0:77..86 Variant { name: Name { span: Span { source: SourceId(0), start: 77, end: 79 } }, payload: [NodeId(7)] }
9 @0:50..89 Enum { name: Name { span: Span { source: SourceId(0), start: 55, end: 60 } }, variants: [NodeId(6), NodeId(8)] }
10 @0:50..89 Item { public: false, declaration: NodeId(9) }
11 @0:104..107 NamedType { path: [Name { span: Span { source: SourceId(0), start: 104, end: 107 } }], arguments: [] }
12 @0:91..108 Alias { name: Name { span: Span { source: SourceId(0), start: 96, end: 101 } }, ty: NodeId(11) }
13 @0:91..108 Item { public: false, declaration: NodeId(12) }
14 @0:123..126 NamedType { path: [Name { span: Span { source: SourceId(0), start: 123, end: 126 } }], arguments: [] }
15 @0:160..161 Literal { kind: Int }
16 @0:157..161 FieldValue { name: Name { span: Span { source: SourceId(0), start: 157, end: 158 } }, value: NodeId(15) }
17 @0:166..167 Literal { kind: Int }
18 @0:163..167 FieldValue { name: Name { span: Span { source: SourceId(0), start: 163, end: 164 } }, value: NodeId(17) }
19 @0:145..169 Construct { path: [Name { span: Span { source: SourceId(0), start: 149, end: 154 } }], fields: [NodeId(16), NodeId(18)] }
20 @0:133..170 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 137, end: 142 } }, ty: None, value: NodeId(19) }
21 @0:187..196 Path { components: [Name { span: Span { source: SourceId(0), start: 187, end: 192 } }, Name { span: Span { source: SourceId(0), start: 194, end: 196 } }] }
22 @0:197..202 Path { components: [Name { span: Span { source: SourceId(0), start: 197, end: 202 } }] }
23 @0:187..203 Call { callee: NodeId(21), arguments: [NodeId(22)] }
24 @0:175..204 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 179, end: 184 } }, ty: None, value: NodeId(23) }
25 @0:215..220 Path { components: [Name { span: Span { source: SourceId(0), start: 215, end: 220 } }] }
26 @0:231..244 ConstructorPattern { path: [Name { span: Span { source: SourceId(0), start: 231, end: 236 } }, Name { span: Span { source: SourceId(0), start: 238, end: 242 } }], fields: [] }
27 @0:257..258 Literal { kind: Int }
28 @0:250..259 Return { value: Some(NodeId(27)) }
29 @0:248..261 Block { statements: [NodeId(28)] }
30 @0:231..261 Arm { pattern: NodeId(26), body: NodeId(29) }
31 @0:280..281 BindingPattern { name: Name { span: Span { source: SourceId(0), start: 280, end: 281 } } }
32 @0:270..282 ConstructorPattern { path: [Name { span: Span { source: SourceId(0), start: 270, end: 275 } }, Name { span: Span { source: SourceId(0), start: 277, end: 279 } }], fields: [NodeId(31)] }
33 @0:300..307 Path { components: [Name { span: Span { source: SourceId(0), start: 300, end: 307 } }] }
34 @0:308..309 Path { components: [Name { span: Span { source: SourceId(0), start: 308, end: 309 } }] }
35 @0:308..311 Member { receiver: NodeId(34), name: Name { span: Span { source: SourceId(0), start: 310, end: 311 } } }
36 @0:314..315 Path { components: [Name { span: Span { source: SourceId(0), start: 314, end: 315 } }] }
37 @0:314..317 Member { receiver: NodeId(36), name: Name { span: Span { source: SourceId(0), start: 316, end: 317 } } }
38 @0:308..317 Binary { operator: Plus, left: NodeId(35), right: NodeId(37) }
39 @0:319..343 Literal { kind: String }
40 @0:300..344 Call { callee: NodeId(33), arguments: [NodeId(38), NodeId(39)] }
41 @0:300..345 ExpressionStatement { value: NodeId(40) }
42 @0:365..366 Literal { kind: Int }
43 @0:358..367 Return { value: Some(NodeId(42)) }
44 @0:286..377 Block { statements: [NodeId(41), NodeId(43)] }
45 @0:270..377 Arm { pattern: NodeId(32), body: NodeId(44) }
46 @0:209..383 Match { subject: NodeId(25), arms: [NodeId(30), NodeId(45)] }
47 @0:127..385 Block { statements: [NodeId(20), NodeId(24), NodeId(46)] }
48 @0:110..385 Function { name: Name { span: Span { source: SourceId(0), start: 113, end: 117 } }, parameters: [], result: NodeId(14), from: None, body: NodeId(47) }
49 @0:110..385 Item { public: false, declaration: NodeId(48) }
50 @0:0..386 Program { module: None, imports: [], items: [NodeId(5), NodeId(10), NodeId(13), NodeId(49)] }
```

</details>

## 08-errors.cretes

```cretes
enum InputError { Negative, }

fn validate(value: i64) -> Result[i64, InputError] {
    if value < 0 {
        return Result::Err(InputError::Negative());
    }
    return Result::Ok(value);
}

fn main() -> Result[i32, InputError] {
    let value = validate(12)?;
    let optional: Option[i64] = Option::Some(value);
    match optional {
        Option::Some(n) => { discard(n, "Validated demonstration input"); }
        Option::None() => { return Result::Ok(1); }
    }
    return Result::Ok(0);
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Enum 0..4 "enum"
Ident 5..15 "InputError"
LBrace 16..17 "{"
Ident 18..26 "Negative"
Comma 26..27 ","
RBrace 28..29 "}"
Fn 31..33 "fn"
Ident 34..42 "validate"
LParen 42..43 "("
Ident 43..48 "value"
Colon 48..49 ":"
Ident 50..53 "i64"
RParen 53..54 ")"
Arrow 55..57 "->"
Ident 58..64 "Result"
LBracket 64..65 "["
Ident 65..68 "i64"
Comma 68..69 ","
Ident 70..80 "InputError"
RBracket 80..81 "]"
LBrace 82..83 "{"
If 88..90 "if"
Ident 91..96 "value"
Lt 97..98 "<"
Int 99..100 "0"
LBrace 101..102 "{"
Return 111..117 "return"
Ident 118..124 "Result"
ColonColon 124..126 "::"
Ident 126..129 "Err"
LParen 129..130 "("
Ident 130..140 "InputError"
ColonColon 140..142 "::"
Ident 142..150 "Negative"
LParen 150..151 "("
RParen 151..152 ")"
RParen 152..153 ")"
Semi 153..154 ";"
RBrace 159..160 "}"
Return 165..171 "return"
Ident 172..178 "Result"
ColonColon 178..180 "::"
Ident 180..182 "Ok"
LParen 182..183 "("
Ident 183..188 "value"
RParen 188..189 ")"
Semi 189..190 ";"
RBrace 191..192 "}"
Fn 194..196 "fn"
Ident 197..201 "main"
LParen 201..202 "("
RParen 202..203 ")"
Arrow 204..206 "->"
Ident 207..213 "Result"
LBracket 213..214 "["
Ident 214..217 "i32"
Comma 217..218 ","
Ident 219..229 "InputError"
RBracket 229..230 "]"
LBrace 231..232 "{"
Let 237..240 "let"
Ident 241..246 "value"
Eq 247..248 "="
Ident 249..257 "validate"
LParen 257..258 "("
Int 258..260 "12"
RParen 260..261 ")"
Question 261..262 "?"
Semi 262..263 ";"
Let 268..271 "let"
Ident 272..280 "optional"
Colon 280..281 ":"
Ident 282..288 "Option"
LBracket 288..289 "["
Ident 289..292 "i64"
RBracket 292..293 "]"
Eq 294..295 "="
Ident 296..302 "Option"
ColonColon 302..304 "::"
Ident 304..308 "Some"
LParen 308..309 "("
Ident 309..314 "value"
RParen 314..315 ")"
Semi 315..316 ";"
Match 321..326 "match"
Ident 327..335 "optional"
LBrace 336..337 "{"
Ident 346..352 "Option"
ColonColon 352..354 "::"
Ident 354..358 "Some"
LParen 358..359 "("
Ident 359..360 "n"
RParen 360..361 ")"
FatArrow 362..364 "=>"
LBrace 365..366 "{"
Ident 367..374 "discard"
LParen 374..375 "("
Ident 375..376 "n"
Comma 376..377 ","
String 378..409 "\"Validated demonstration input\""
RParen 409..410 ")"
Semi 410..411 ";"
RBrace 412..413 "}"
Ident 422..428 "Option"
ColonColon 428..430 "::"
Ident 430..434 "None"
LParen 434..435 "("
RParen 435..436 ")"
FatArrow 437..439 "=>"
LBrace 440..441 "{"
Return 442..448 "return"
Ident 449..455 "Result"
ColonColon 455..457 "::"
Ident 457..459 "Ok"
LParen 459..460 "("
Int 460..461 "1"
RParen 461..462 ")"
Semi 462..463 ";"
RBrace 464..465 "}"
RBrace 470..471 "}"
Return 476..482 "return"
Ident 483..489 "Result"
ColonColon 489..491 "::"
Ident 491..493 "Ok"
LParen 493..494 "("
Int 494..495 "0"
RParen 495..496 ")"
Semi 496..497 ";"
RBrace 498..499 "}"
Eof 500..500 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:18..26 Variant { name: Name { span: Span { source: SourceId(0), start: 18, end: 26 } }, payload: [] }
1 @0:0..29 Enum { name: Name { span: Span { source: SourceId(0), start: 5, end: 15 } }, variants: [NodeId(0)] }
2 @0:0..29 Item { public: false, declaration: NodeId(1) }
3 @0:50..53 NamedType { path: [Name { span: Span { source: SourceId(0), start: 50, end: 53 } }], arguments: [] }
4 @0:43..53 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 43, end: 48 } }, ty: NodeId(3) }
5 @0:65..68 NamedType { path: [Name { span: Span { source: SourceId(0), start: 65, end: 68 } }], arguments: [] }
6 @0:70..80 NamedType { path: [Name { span: Span { source: SourceId(0), start: 70, end: 80 } }], arguments: [] }
7 @0:58..81 NamedType { path: [Name { span: Span { source: SourceId(0), start: 58, end: 64 } }], arguments: [NodeId(5), NodeId(6)] }
8 @0:91..96 Path { components: [Name { span: Span { source: SourceId(0), start: 91, end: 96 } }] }
9 @0:99..100 Literal { kind: Int }
10 @0:91..100 Binary { operator: Lt, left: NodeId(8), right: NodeId(9) }
11 @0:118..129 Path { components: [Name { span: Span { source: SourceId(0), start: 118, end: 124 } }, Name { span: Span { source: SourceId(0), start: 126, end: 129 } }] }
12 @0:130..150 Path { components: [Name { span: Span { source: SourceId(0), start: 130, end: 140 } }, Name { span: Span { source: SourceId(0), start: 142, end: 150 } }] }
13 @0:130..152 Call { callee: NodeId(12), arguments: [] }
14 @0:118..153 Call { callee: NodeId(11), arguments: [NodeId(13)] }
15 @0:111..154 Return { value: Some(NodeId(14)) }
16 @0:101..160 Block { statements: [NodeId(15)] }
17 @0:88..160 If { condition: NodeId(10), then_block: NodeId(16), otherwise: None }
18 @0:172..182 Path { components: [Name { span: Span { source: SourceId(0), start: 172, end: 178 } }, Name { span: Span { source: SourceId(0), start: 180, end: 182 } }] }
19 @0:183..188 Path { components: [Name { span: Span { source: SourceId(0), start: 183, end: 188 } }] }
20 @0:172..189 Call { callee: NodeId(18), arguments: [NodeId(19)] }
21 @0:165..190 Return { value: Some(NodeId(20)) }
22 @0:82..192 Block { statements: [NodeId(17), NodeId(21)] }
23 @0:31..192 Function { name: Name { span: Span { source: SourceId(0), start: 34, end: 42 } }, parameters: [NodeId(4)], result: NodeId(7), from: None, body: NodeId(22) }
24 @0:31..192 Item { public: false, declaration: NodeId(23) }
25 @0:214..217 NamedType { path: [Name { span: Span { source: SourceId(0), start: 214, end: 217 } }], arguments: [] }
26 @0:219..229 NamedType { path: [Name { span: Span { source: SourceId(0), start: 219, end: 229 } }], arguments: [] }
27 @0:207..230 NamedType { path: [Name { span: Span { source: SourceId(0), start: 207, end: 213 } }], arguments: [NodeId(25), NodeId(26)] }
28 @0:249..257 Path { components: [Name { span: Span { source: SourceId(0), start: 249, end: 257 } }] }
29 @0:258..260 Literal { kind: Int }
30 @0:249..261 Call { callee: NodeId(28), arguments: [NodeId(29)] }
31 @0:249..262 Propagate { value: NodeId(30) }
32 @0:237..263 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 241, end: 246 } }, ty: None, value: NodeId(31) }
33 @0:289..292 NamedType { path: [Name { span: Span { source: SourceId(0), start: 289, end: 292 } }], arguments: [] }
34 @0:282..293 NamedType { path: [Name { span: Span { source: SourceId(0), start: 282, end: 288 } }], arguments: [NodeId(33)] }
35 @0:296..308 Path { components: [Name { span: Span { source: SourceId(0), start: 296, end: 302 } }, Name { span: Span { source: SourceId(0), start: 304, end: 308 } }] }
36 @0:309..314 Path { components: [Name { span: Span { source: SourceId(0), start: 309, end: 314 } }] }
37 @0:296..315 Call { callee: NodeId(35), arguments: [NodeId(36)] }
38 @0:268..316 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 272, end: 280 } }, ty: Some(NodeId(34)), value: NodeId(37) }
39 @0:327..335 Path { components: [Name { span: Span { source: SourceId(0), start: 327, end: 335 } }] }
40 @0:359..360 BindingPattern { name: Name { span: Span { source: SourceId(0), start: 359, end: 360 } } }
41 @0:346..361 ConstructorPattern { path: [Name { span: Span { source: SourceId(0), start: 346, end: 352 } }, Name { span: Span { source: SourceId(0), start: 354, end: 358 } }], fields: [NodeId(40)] }
42 @0:367..374 Path { components: [Name { span: Span { source: SourceId(0), start: 367, end: 374 } }] }
43 @0:375..376 Path { components: [Name { span: Span { source: SourceId(0), start: 375, end: 376 } }] }
44 @0:378..409 Literal { kind: String }
45 @0:367..410 Call { callee: NodeId(42), arguments: [NodeId(43), NodeId(44)] }
46 @0:367..411 ExpressionStatement { value: NodeId(45) }
47 @0:365..413 Block { statements: [NodeId(46)] }
48 @0:346..413 Arm { pattern: NodeId(41), body: NodeId(47) }
49 @0:422..436 ConstructorPattern { path: [Name { span: Span { source: SourceId(0), start: 422, end: 428 } }, Name { span: Span { source: SourceId(0), start: 430, end: 434 } }], fields: [] }
50 @0:449..459 Path { components: [Name { span: Span { source: SourceId(0), start: 449, end: 455 } }, Name { span: Span { source: SourceId(0), start: 457, end: 459 } }] }
51 @0:460..461 Literal { kind: Int }
52 @0:449..462 Call { callee: NodeId(50), arguments: [NodeId(51)] }
53 @0:442..463 Return { value: Some(NodeId(52)) }
54 @0:440..465 Block { statements: [NodeId(53)] }
55 @0:422..465 Arm { pattern: NodeId(49), body: NodeId(54) }
56 @0:321..471 Match { subject: NodeId(39), arms: [NodeId(48), NodeId(55)] }
57 @0:483..493 Path { components: [Name { span: Span { source: SourceId(0), start: 483, end: 489 } }, Name { span: Span { source: SourceId(0), start: 491, end: 493 } }] }
58 @0:494..495 Literal { kind: Int }
59 @0:483..496 Call { callee: NodeId(57), arguments: [NodeId(58)] }
60 @0:476..497 Return { value: Some(NodeId(59)) }
61 @0:231..499 Block { statements: [NodeId(32), NodeId(38), NodeId(56), NodeId(60)] }
62 @0:194..499 Function { name: Name { span: Span { source: SourceId(0), start: 197, end: 201 } }, parameters: [], result: NodeId(27), from: None, body: NodeId(61) }
63 @0:194..499 Item { public: false, declaration: NodeId(62) }
64 @0:0..500 Program { module: None, imports: [], items: [NodeId(2), NodeId(24), NodeId(63)] }
```

</details>

## 09-modules.cretes

```cretes
module demo::math;

pub fn twice(value: i64) -> i64 {
    return value * 2;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Module 0..6 "module"
Ident 7..11 "demo"
ColonColon 11..13 "::"
Ident 13..17 "math"
Semi 17..18 ";"
Pub 20..23 "pub"
Fn 24..26 "fn"
Ident 27..32 "twice"
LParen 32..33 "("
Ident 33..38 "value"
Colon 38..39 ":"
Ident 40..43 "i64"
RParen 43..44 ")"
Arrow 45..47 "->"
Ident 48..51 "i64"
LBrace 52..53 "{"
Return 58..64 "return"
Ident 65..70 "value"
Star 71..72 "*"
Int 73..74 "2"
Semi 74..75 ";"
RBrace 76..77 "}"
Eof 78..78 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:0..18 Module { path: [Name { span: Span { source: SourceId(0), start: 7, end: 11 } }, Name { span: Span { source: SourceId(0), start: 13, end: 17 } }] }
1 @0:40..43 NamedType { path: [Name { span: Span { source: SourceId(0), start: 40, end: 43 } }], arguments: [] }
2 @0:33..43 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 33, end: 38 } }, ty: NodeId(1) }
3 @0:48..51 NamedType { path: [Name { span: Span { source: SourceId(0), start: 48, end: 51 } }], arguments: [] }
4 @0:65..70 Path { components: [Name { span: Span { source: SourceId(0), start: 65, end: 70 } }] }
5 @0:73..74 Literal { kind: Int }
6 @0:65..74 Binary { operator: Star, left: NodeId(4), right: NodeId(5) }
7 @0:58..75 Return { value: Some(NodeId(6)) }
8 @0:52..77 Block { statements: [NodeId(7)] }
9 @0:24..77 Function { name: Name { span: Span { source: SourceId(0), start: 27, end: 32 } }, parameters: [NodeId(2)], result: NodeId(3), from: None, body: NodeId(8) }
10 @0:20..77 Item { public: true, declaration: NodeId(9) }
11 @0:0..78 Program { module: Some(NodeId(0)), imports: [], items: [NodeId(10)] }
```

</details>

## 10-borrowing.cretes

```cretes
fn first(values: &Seq[i64]) -> &i64 from values {
    return &(*values)[0];
}

fn increment(value: &mut i64) -> () {
    *value = *value + 1;
}

fn main() -> i32 {
    var count: i64 = 0;
    increment(&mut count);
    let values: Seq[i64] = [count];
    let view = first(&values);
    discard(*view, "Inspect the borrowed value before its owner exits");
    return 0;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Fn 0..2 "fn"
Ident 3..8 "first"
LParen 8..9 "("
Ident 9..15 "values"
Colon 15..16 ":"
Amp 17..18 "&"
Ident 18..21 "Seq"
LBracket 21..22 "["
Ident 22..25 "i64"
RBracket 25..26 "]"
RParen 26..27 ")"
Arrow 28..30 "->"
Amp 31..32 "&"
Ident 32..35 "i64"
From 36..40 "from"
Ident 41..47 "values"
LBrace 48..49 "{"
Return 54..60 "return"
Amp 61..62 "&"
LParen 62..63 "("
Star 63..64 "*"
Ident 64..70 "values"
RParen 70..71 ")"
LBracket 71..72 "["
Int 72..73 "0"
RBracket 73..74 "]"
Semi 74..75 ";"
RBrace 76..77 "}"
Fn 79..81 "fn"
Ident 82..91 "increment"
LParen 91..92 "("
Ident 92..97 "value"
Colon 97..98 ":"
Amp 99..100 "&"
Mut 100..103 "mut"
Ident 104..107 "i64"
RParen 107..108 ")"
Arrow 109..111 "->"
LParen 112..113 "("
RParen 113..114 ")"
LBrace 115..116 "{"
Star 121..122 "*"
Ident 122..127 "value"
Eq 128..129 "="
Star 130..131 "*"
Ident 131..136 "value"
Plus 137..138 "+"
Int 139..140 "1"
Semi 140..141 ";"
RBrace 142..143 "}"
Fn 145..147 "fn"
Ident 148..152 "main"
LParen 152..153 "("
RParen 153..154 ")"
Arrow 155..157 "->"
Ident 158..161 "i32"
LBrace 162..163 "{"
Var 168..171 "var"
Ident 172..177 "count"
Colon 177..178 ":"
Ident 179..182 "i64"
Eq 183..184 "="
Int 185..186 "0"
Semi 186..187 ";"
Ident 192..201 "increment"
LParen 201..202 "("
Amp 202..203 "&"
Mut 203..206 "mut"
Ident 207..212 "count"
RParen 212..213 ")"
Semi 213..214 ";"
Let 219..222 "let"
Ident 223..229 "values"
Colon 229..230 ":"
Ident 231..234 "Seq"
LBracket 234..235 "["
Ident 235..238 "i64"
RBracket 238..239 "]"
Eq 240..241 "="
LBracket 242..243 "["
Ident 243..248 "count"
RBracket 248..249 "]"
Semi 249..250 ";"
Let 255..258 "let"
Ident 259..263 "view"
Eq 264..265 "="
Ident 266..271 "first"
LParen 271..272 "("
Amp 272..273 "&"
Ident 273..279 "values"
RParen 279..280 ")"
Semi 280..281 ";"
Ident 286..293 "discard"
LParen 293..294 "("
Star 294..295 "*"
Ident 295..299 "view"
Comma 299..300 ","
String 301..352 "\"Inspect the borrowed value before its owner exits\""
RParen 352..353 ")"
Semi 353..354 ";"
Return 359..365 "return"
Int 366..367 "0"
Semi 367..368 ";"
RBrace 369..370 "}"
Eof 371..371 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:22..25 NamedType { path: [Name { span: Span { source: SourceId(0), start: 22, end: 25 } }], arguments: [] }
1 @0:18..26 NamedType { path: [Name { span: Span { source: SourceId(0), start: 18, end: 21 } }], arguments: [NodeId(0)] }
2 @0:17..26 ReferenceType { mutable: false, target: NodeId(1) }
3 @0:9..26 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 9, end: 15 } }, ty: NodeId(2) }
4 @0:32..35 NamedType { path: [Name { span: Span { source: SourceId(0), start: 32, end: 35 } }], arguments: [] }
5 @0:31..35 ReferenceType { mutable: false, target: NodeId(4) }
6 @0:64..70 Path { components: [Name { span: Span { source: SourceId(0), start: 64, end: 70 } }] }
7 @0:63..70 Unary { operator: Star, mutable: false, operand: NodeId(6) }
8 @0:62..71 Group { value: NodeId(7) }
9 @0:72..73 Literal { kind: Int }
10 @0:62..74 Index { receiver: NodeId(8), index: NodeId(9) }
11 @0:61..74 Unary { operator: Amp, mutable: false, operand: NodeId(10) }
12 @0:54..75 Return { value: Some(NodeId(11)) }
13 @0:48..77 Block { statements: [NodeId(12)] }
14 @0:0..77 Function { name: Name { span: Span { source: SourceId(0), start: 3, end: 8 } }, parameters: [NodeId(3)], result: NodeId(5), from: Some(Name { span: Span { source: SourceId(0), start: 41, end: 47 } }), body: NodeId(13) }
15 @0:0..77 Item { public: false, declaration: NodeId(14) }
16 @0:104..107 NamedType { path: [Name { span: Span { source: SourceId(0), start: 104, end: 107 } }], arguments: [] }
17 @0:99..107 ReferenceType { mutable: true, target: NodeId(16) }
18 @0:92..107 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 92, end: 97 } }, ty: NodeId(17) }
19 @0:112..114 TupleType { elements: [] }
20 @0:122..127 Path { components: [Name { span: Span { source: SourceId(0), start: 122, end: 127 } }] }
21 @0:121..127 Unary { operator: Star, mutable: false, operand: NodeId(20) }
22 @0:131..136 Path { components: [Name { span: Span { source: SourceId(0), start: 131, end: 136 } }] }
23 @0:130..136 Unary { operator: Star, mutable: false, operand: NodeId(22) }
24 @0:139..140 Literal { kind: Int }
25 @0:130..140 Binary { operator: Plus, left: NodeId(23), right: NodeId(24) }
26 @0:121..141 Assignment { place: NodeId(21), value: NodeId(25) }
27 @0:115..143 Block { statements: [NodeId(26)] }
28 @0:79..143 Function { name: Name { span: Span { source: SourceId(0), start: 82, end: 91 } }, parameters: [NodeId(18)], result: NodeId(19), from: None, body: NodeId(27) }
29 @0:79..143 Item { public: false, declaration: NodeId(28) }
30 @0:158..161 NamedType { path: [Name { span: Span { source: SourceId(0), start: 158, end: 161 } }], arguments: [] }
31 @0:179..182 NamedType { path: [Name { span: Span { source: SourceId(0), start: 179, end: 182 } }], arguments: [] }
32 @0:185..186 Literal { kind: Int }
33 @0:168..187 Binding { mutable: true, name: Name { span: Span { source: SourceId(0), start: 172, end: 177 } }, ty: Some(NodeId(31)), value: NodeId(32) }
34 @0:192..201 Path { components: [Name { span: Span { source: SourceId(0), start: 192, end: 201 } }] }
35 @0:207..212 Path { components: [Name { span: Span { source: SourceId(0), start: 207, end: 212 } }] }
36 @0:202..212 Unary { operator: Amp, mutable: true, operand: NodeId(35) }
37 @0:192..213 Call { callee: NodeId(34), arguments: [NodeId(36)] }
38 @0:192..214 ExpressionStatement { value: NodeId(37) }
39 @0:235..238 NamedType { path: [Name { span: Span { source: SourceId(0), start: 235, end: 238 } }], arguments: [] }
40 @0:231..239 NamedType { path: [Name { span: Span { source: SourceId(0), start: 231, end: 234 } }], arguments: [NodeId(39)] }
41 @0:243..248 Path { components: [Name { span: Span { source: SourceId(0), start: 243, end: 248 } }] }
42 @0:242..249 Sequence { elements: [NodeId(41)] }
43 @0:219..250 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 223, end: 229 } }, ty: Some(NodeId(40)), value: NodeId(42) }
44 @0:266..271 Path { components: [Name { span: Span { source: SourceId(0), start: 266, end: 271 } }] }
45 @0:273..279 Path { components: [Name { span: Span { source: SourceId(0), start: 273, end: 279 } }] }
46 @0:272..279 Unary { operator: Amp, mutable: false, operand: NodeId(45) }
47 @0:266..280 Call { callee: NodeId(44), arguments: [NodeId(46)] }
48 @0:255..281 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 259, end: 263 } }, ty: None, value: NodeId(47) }
49 @0:286..293 Path { components: [Name { span: Span { source: SourceId(0), start: 286, end: 293 } }] }
50 @0:295..299 Path { components: [Name { span: Span { source: SourceId(0), start: 295, end: 299 } }] }
51 @0:294..299 Unary { operator: Star, mutable: false, operand: NodeId(50) }
52 @0:301..352 Literal { kind: String }
53 @0:286..353 Call { callee: NodeId(49), arguments: [NodeId(51), NodeId(52)] }
54 @0:286..354 ExpressionStatement { value: NodeId(53) }
55 @0:366..367 Literal { kind: Int }
56 @0:359..368 Return { value: Some(NodeId(55)) }
57 @0:162..370 Block { statements: [NodeId(33), NodeId(38), NodeId(43), NodeId(48), NodeId(54), NodeId(56)] }
58 @0:145..370 Function { name: Name { span: Span { source: SourceId(0), start: 148, end: 152 } }, parameters: [], result: NodeId(30), from: None, body: NodeId(57) }
59 @0:145..370 Item { public: false, declaration: NodeId(58) }
60 @0:0..371 Program { module: None, imports: [], items: [NodeId(15), NodeId(29), NodeId(59)] }
```

</details>

## 11-automation.cretes

```cretes
import std::fs;

fn main() -> Result[i32, fs::Error] {
    let data = fs::read_bytes("input.bin")?;
    fs::write_bytes("output.bin", &data)?;
    return Result::Ok(0);
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Import 0..6 "import"
Ident 7..10 "std"
ColonColon 10..12 "::"
Ident 12..14 "fs"
Semi 14..15 ";"
Fn 17..19 "fn"
Ident 20..24 "main"
LParen 24..25 "("
RParen 25..26 ")"
Arrow 27..29 "->"
Ident 30..36 "Result"
LBracket 36..37 "["
Ident 37..40 "i32"
Comma 40..41 ","
Ident 42..44 "fs"
ColonColon 44..46 "::"
Ident 46..51 "Error"
RBracket 51..52 "]"
LBrace 53..54 "{"
Let 59..62 "let"
Ident 63..67 "data"
Eq 68..69 "="
Ident 70..72 "fs"
ColonColon 72..74 "::"
Ident 74..84 "read_bytes"
LParen 84..85 "("
String 85..96 "\"input.bin\""
RParen 96..97 ")"
Question 97..98 "?"
Semi 98..99 ";"
Ident 104..106 "fs"
ColonColon 106..108 "::"
Ident 108..119 "write_bytes"
LParen 119..120 "("
String 120..132 "\"output.bin\""
Comma 132..133 ","
Amp 134..135 "&"
Ident 135..139 "data"
RParen 139..140 ")"
Question 140..141 "?"
Semi 141..142 ";"
Return 147..153 "return"
Ident 154..160 "Result"
ColonColon 160..162 "::"
Ident 162..164 "Ok"
LParen 164..165 "("
Int 165..166 "0"
RParen 166..167 ")"
Semi 167..168 ";"
RBrace 169..170 "}"
Eof 171..171 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:0..15 Import { path: [Name { span: Span { source: SourceId(0), start: 7, end: 10 } }, Name { span: Span { source: SourceId(0), start: 12, end: 14 } }], alias: None }
1 @0:37..40 NamedType { path: [Name { span: Span { source: SourceId(0), start: 37, end: 40 } }], arguments: [] }
2 @0:42..51 NamedType { path: [Name { span: Span { source: SourceId(0), start: 42, end: 44 } }, Name { span: Span { source: SourceId(0), start: 46, end: 51 } }], arguments: [] }
3 @0:30..52 NamedType { path: [Name { span: Span { source: SourceId(0), start: 30, end: 36 } }], arguments: [NodeId(1), NodeId(2)] }
4 @0:70..84 Path { components: [Name { span: Span { source: SourceId(0), start: 70, end: 72 } }, Name { span: Span { source: SourceId(0), start: 74, end: 84 } }] }
5 @0:85..96 Literal { kind: String }
6 @0:70..97 Call { callee: NodeId(4), arguments: [NodeId(5)] }
7 @0:70..98 Propagate { value: NodeId(6) }
8 @0:59..99 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 63, end: 67 } }, ty: None, value: NodeId(7) }
9 @0:104..119 Path { components: [Name { span: Span { source: SourceId(0), start: 104, end: 106 } }, Name { span: Span { source: SourceId(0), start: 108, end: 119 } }] }
10 @0:120..132 Literal { kind: String }
11 @0:135..139 Path { components: [Name { span: Span { source: SourceId(0), start: 135, end: 139 } }] }
12 @0:134..139 Unary { operator: Amp, mutable: false, operand: NodeId(11) }
13 @0:104..140 Call { callee: NodeId(9), arguments: [NodeId(10), NodeId(12)] }
14 @0:104..141 Propagate { value: NodeId(13) }
15 @0:104..142 ExpressionStatement { value: NodeId(14) }
16 @0:154..164 Path { components: [Name { span: Span { source: SourceId(0), start: 154, end: 160 } }, Name { span: Span { source: SourceId(0), start: 162, end: 164 } }] }
17 @0:165..166 Literal { kind: Int }
18 @0:154..167 Call { callee: NodeId(16), arguments: [NodeId(17)] }
19 @0:147..168 Return { value: Some(NodeId(18)) }
20 @0:53..170 Block { statements: [NodeId(8), NodeId(15), NodeId(19)] }
21 @0:17..170 Function { name: Name { span: Span { source: SourceId(0), start: 20, end: 24 } }, parameters: [], result: NodeId(3), from: None, body: NodeId(20) }
22 @0:17..170 Item { public: false, declaration: NodeId(21) }
23 @0:0..171 Program { module: None, imports: [NodeId(0)], items: [NodeId(22)] }
```

</details>

## 12-packet-validation.cretes

```cretes
import std::bytes;

enum PacketError { Truncated, Unsupported, }

fn version(packet: &Bytes) -> Result[u8, PacketError] {
    if bytes::len(packet) < 1 {
        return Result::Err(PacketError::Truncated());
    }
    let value: u8 = (*packet)[0];
    if value != 1 {
        return Result::Err(PacketError::Unsupported());
    }
    return Result::Ok(value);
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Import 0..6 "import"
Ident 7..10 "std"
ColonColon 10..12 "::"
Ident 12..17 "bytes"
Semi 17..18 ";"
Enum 20..24 "enum"
Ident 25..36 "PacketError"
LBrace 37..38 "{"
Ident 39..48 "Truncated"
Comma 48..49 ","
Ident 50..61 "Unsupported"
Comma 61..62 ","
RBrace 63..64 "}"
Fn 66..68 "fn"
Ident 69..76 "version"
LParen 76..77 "("
Ident 77..83 "packet"
Colon 83..84 ":"
Amp 85..86 "&"
Ident 86..91 "Bytes"
RParen 91..92 ")"
Arrow 93..95 "->"
Ident 96..102 "Result"
LBracket 102..103 "["
Ident 103..105 "u8"
Comma 105..106 ","
Ident 107..118 "PacketError"
RBracket 118..119 "]"
LBrace 120..121 "{"
If 126..128 "if"
Ident 129..134 "bytes"
ColonColon 134..136 "::"
Ident 136..139 "len"
LParen 139..140 "("
Ident 140..146 "packet"
RParen 146..147 ")"
Lt 148..149 "<"
Int 150..151 "1"
LBrace 152..153 "{"
Return 162..168 "return"
Ident 169..175 "Result"
ColonColon 175..177 "::"
Ident 177..180 "Err"
LParen 180..181 "("
Ident 181..192 "PacketError"
ColonColon 192..194 "::"
Ident 194..203 "Truncated"
LParen 203..204 "("
RParen 204..205 ")"
RParen 205..206 ")"
Semi 206..207 ";"
RBrace 212..213 "}"
Let 218..221 "let"
Ident 222..227 "value"
Colon 227..228 ":"
Ident 229..231 "u8"
Eq 232..233 "="
LParen 234..235 "("
Star 235..236 "*"
Ident 236..242 "packet"
RParen 242..243 ")"
LBracket 243..244 "["
Int 244..245 "0"
RBracket 245..246 "]"
Semi 246..247 ";"
If 252..254 "if"
Ident 255..260 "value"
NotEq 261..263 "!="
Int 264..265 "1"
LBrace 266..267 "{"
Return 276..282 "return"
Ident 283..289 "Result"
ColonColon 289..291 "::"
Ident 291..294 "Err"
LParen 294..295 "("
Ident 295..306 "PacketError"
ColonColon 306..308 "::"
Ident 308..319 "Unsupported"
LParen 319..320 "("
RParen 320..321 ")"
RParen 321..322 ")"
Semi 322..323 ";"
RBrace 328..329 "}"
Return 334..340 "return"
Ident 341..347 "Result"
ColonColon 347..349 "::"
Ident 349..351 "Ok"
LParen 351..352 "("
Ident 352..357 "value"
RParen 357..358 ")"
Semi 358..359 ";"
RBrace 360..361 "}"
Eof 362..362 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:0..18 Import { path: [Name { span: Span { source: SourceId(0), start: 7, end: 10 } }, Name { span: Span { source: SourceId(0), start: 12, end: 17 } }], alias: None }
1 @0:39..48 Variant { name: Name { span: Span { source: SourceId(0), start: 39, end: 48 } }, payload: [] }
2 @0:50..61 Variant { name: Name { span: Span { source: SourceId(0), start: 50, end: 61 } }, payload: [] }
3 @0:20..64 Enum { name: Name { span: Span { source: SourceId(0), start: 25, end: 36 } }, variants: [NodeId(1), NodeId(2)] }
4 @0:20..64 Item { public: false, declaration: NodeId(3) }
5 @0:86..91 NamedType { path: [Name { span: Span { source: SourceId(0), start: 86, end: 91 } }], arguments: [] }
6 @0:85..91 ReferenceType { mutable: false, target: NodeId(5) }
7 @0:77..91 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 77, end: 83 } }, ty: NodeId(6) }
8 @0:103..105 NamedType { path: [Name { span: Span { source: SourceId(0), start: 103, end: 105 } }], arguments: [] }
9 @0:107..118 NamedType { path: [Name { span: Span { source: SourceId(0), start: 107, end: 118 } }], arguments: [] }
10 @0:96..119 NamedType { path: [Name { span: Span { source: SourceId(0), start: 96, end: 102 } }], arguments: [NodeId(8), NodeId(9)] }
11 @0:129..139 Path { components: [Name { span: Span { source: SourceId(0), start: 129, end: 134 } }, Name { span: Span { source: SourceId(0), start: 136, end: 139 } }] }
12 @0:140..146 Path { components: [Name { span: Span { source: SourceId(0), start: 140, end: 146 } }] }
13 @0:129..147 Call { callee: NodeId(11), arguments: [NodeId(12)] }
14 @0:150..151 Literal { kind: Int }
15 @0:129..151 Binary { operator: Lt, left: NodeId(13), right: NodeId(14) }
16 @0:169..180 Path { components: [Name { span: Span { source: SourceId(0), start: 169, end: 175 } }, Name { span: Span { source: SourceId(0), start: 177, end: 180 } }] }
17 @0:181..203 Path { components: [Name { span: Span { source: SourceId(0), start: 181, end: 192 } }, Name { span: Span { source: SourceId(0), start: 194, end: 203 } }] }
18 @0:181..205 Call { callee: NodeId(17), arguments: [] }
19 @0:169..206 Call { callee: NodeId(16), arguments: [NodeId(18)] }
20 @0:162..207 Return { value: Some(NodeId(19)) }
21 @0:152..213 Block { statements: [NodeId(20)] }
22 @0:126..213 If { condition: NodeId(15), then_block: NodeId(21), otherwise: None }
23 @0:229..231 NamedType { path: [Name { span: Span { source: SourceId(0), start: 229, end: 231 } }], arguments: [] }
24 @0:236..242 Path { components: [Name { span: Span { source: SourceId(0), start: 236, end: 242 } }] }
25 @0:235..242 Unary { operator: Star, mutable: false, operand: NodeId(24) }
26 @0:234..243 Group { value: NodeId(25) }
27 @0:244..245 Literal { kind: Int }
28 @0:234..246 Index { receiver: NodeId(26), index: NodeId(27) }
29 @0:218..247 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 222, end: 227 } }, ty: Some(NodeId(23)), value: NodeId(28) }
30 @0:255..260 Path { components: [Name { span: Span { source: SourceId(0), start: 255, end: 260 } }] }
31 @0:264..265 Literal { kind: Int }
32 @0:255..265 Binary { operator: NotEq, left: NodeId(30), right: NodeId(31) }
33 @0:283..294 Path { components: [Name { span: Span { source: SourceId(0), start: 283, end: 289 } }, Name { span: Span { source: SourceId(0), start: 291, end: 294 } }] }
34 @0:295..319 Path { components: [Name { span: Span { source: SourceId(0), start: 295, end: 306 } }, Name { span: Span { source: SourceId(0), start: 308, end: 319 } }] }
35 @0:295..321 Call { callee: NodeId(34), arguments: [] }
36 @0:283..322 Call { callee: NodeId(33), arguments: [NodeId(35)] }
37 @0:276..323 Return { value: Some(NodeId(36)) }
38 @0:266..329 Block { statements: [NodeId(37)] }
39 @0:252..329 If { condition: NodeId(32), then_block: NodeId(38), otherwise: None }
40 @0:341..351 Path { components: [Name { span: Span { source: SourceId(0), start: 341, end: 347 } }, Name { span: Span { source: SourceId(0), start: 349, end: 351 } }] }
41 @0:352..357 Path { components: [Name { span: Span { source: SourceId(0), start: 352, end: 357 } }] }
42 @0:341..358 Call { callee: NodeId(40), arguments: [NodeId(41)] }
43 @0:334..359 Return { value: Some(NodeId(42)) }
44 @0:120..361 Block { statements: [NodeId(22), NodeId(29), NodeId(39), NodeId(43)] }
45 @0:66..361 Function { name: Name { span: Span { source: SourceId(0), start: 69, end: 76 } }, parameters: [NodeId(7)], result: NodeId(10), from: None, body: NodeId(44) }
46 @0:66..361 Item { public: false, declaration: NodeId(45) }
47 @0:0..362 Program { module: None, imports: [NodeId(0)], items: [NodeId(4), NodeId(46)] }
```

</details>

## 13-numeric-preprocessing.cretes

```cretes
fn sum(values: &Seq[f64]) -> f64 {
    var total: f64 = 0.0;
    for value in values {
        total = total + *value;
    }
    return total;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Fn 0..2 "fn"
Ident 3..6 "sum"
LParen 6..7 "("
Ident 7..13 "values"
Colon 13..14 ":"
Amp 15..16 "&"
Ident 16..19 "Seq"
LBracket 19..20 "["
Ident 20..23 "f64"
RBracket 23..24 "]"
RParen 24..25 ")"
Arrow 26..28 "->"
Ident 29..32 "f64"
LBrace 33..34 "{"
Var 39..42 "var"
Ident 43..48 "total"
Colon 48..49 ":"
Ident 50..53 "f64"
Eq 54..55 "="
Float 56..59 "0.0"
Semi 59..60 ";"
For 65..68 "for"
Ident 69..74 "value"
In 75..77 "in"
Ident 78..84 "values"
LBrace 85..86 "{"
Ident 95..100 "total"
Eq 101..102 "="
Ident 103..108 "total"
Plus 109..110 "+"
Star 111..112 "*"
Ident 112..117 "value"
Semi 117..118 ";"
RBrace 123..124 "}"
Return 129..135 "return"
Ident 136..141 "total"
Semi 141..142 ";"
RBrace 143..144 "}"
Eof 145..145 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:20..23 NamedType { path: [Name { span: Span { source: SourceId(0), start: 20, end: 23 } }], arguments: [] }
1 @0:16..24 NamedType { path: [Name { span: Span { source: SourceId(0), start: 16, end: 19 } }], arguments: [NodeId(0)] }
2 @0:15..24 ReferenceType { mutable: false, target: NodeId(1) }
3 @0:7..24 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 7, end: 13 } }, ty: NodeId(2) }
4 @0:29..32 NamedType { path: [Name { span: Span { source: SourceId(0), start: 29, end: 32 } }], arguments: [] }
5 @0:50..53 NamedType { path: [Name { span: Span { source: SourceId(0), start: 50, end: 53 } }], arguments: [] }
6 @0:56..59 Literal { kind: Float }
7 @0:39..60 Binding { mutable: true, name: Name { span: Span { source: SourceId(0), start: 43, end: 48 } }, ty: Some(NodeId(5)), value: NodeId(6) }
8 @0:78..84 Path { components: [Name { span: Span { source: SourceId(0), start: 78, end: 84 } }] }
9 @0:95..100 Path { components: [Name { span: Span { source: SourceId(0), start: 95, end: 100 } }] }
10 @0:103..108 Path { components: [Name { span: Span { source: SourceId(0), start: 103, end: 108 } }] }
11 @0:112..117 Path { components: [Name { span: Span { source: SourceId(0), start: 112, end: 117 } }] }
12 @0:111..117 Unary { operator: Star, mutable: false, operand: NodeId(11) }
13 @0:103..117 Binary { operator: Plus, left: NodeId(10), right: NodeId(12) }
14 @0:95..118 Assignment { place: NodeId(9), value: NodeId(13) }
15 @0:85..124 Block { statements: [NodeId(14)] }
16 @0:65..124 For { name: Name { span: Span { source: SourceId(0), start: 69, end: 74 } }, iterable: NodeId(8), body: NodeId(15) }
17 @0:136..141 Path { components: [Name { span: Span { source: SourceId(0), start: 136, end: 141 } }] }
18 @0:129..142 Return { value: Some(NodeId(17)) }
19 @0:33..144 Block { statements: [NodeId(7), NodeId(16), NodeId(18)] }
20 @0:0..144 Function { name: Name { span: Span { source: SourceId(0), start: 3, end: 6 } }, parameters: [NodeId(3)], result: NodeId(4), from: None, body: NodeId(19) }
21 @0:0..144 Item { public: false, declaration: NodeId(20) }
22 @0:0..145 Program { module: None, imports: [], items: [NodeId(21)] }
```

</details>

## 14-defensive-bytes.cretes

```cretes
import std::bytes;

fn has_magic(data: &Bytes) -> bool {
    if bytes::len(data) < 2 {
        return false;
    }
    let first: u8 = (*data)[0];
    let second: u8 = (*data)[1];
    return first == 0x43 && second == 0x52;
}
```

<details><summary>lex: actual output, exit 0</summary>

```text
Import 0..6 "import"
Ident 7..10 "std"
ColonColon 10..12 "::"
Ident 12..17 "bytes"
Semi 17..18 ";"
Fn 20..22 "fn"
Ident 23..32 "has_magic"
LParen 32..33 "("
Ident 33..37 "data"
Colon 37..38 ":"
Amp 39..40 "&"
Ident 40..45 "Bytes"
RParen 45..46 ")"
Arrow 47..49 "->"
Ident 50..54 "bool"
LBrace 55..56 "{"
If 61..63 "if"
Ident 64..69 "bytes"
ColonColon 69..71 "::"
Ident 71..74 "len"
LParen 74..75 "("
Ident 75..79 "data"
RParen 79..80 ")"
Lt 81..82 "<"
Int 83..84 "2"
LBrace 85..86 "{"
Return 95..101 "return"
False 102..107 "false"
Semi 107..108 ";"
RBrace 113..114 "}"
Let 119..122 "let"
Ident 123..128 "first"
Colon 128..129 ":"
Ident 130..132 "u8"
Eq 133..134 "="
LParen 135..136 "("
Star 136..137 "*"
Ident 137..141 "data"
RParen 141..142 ")"
LBracket 142..143 "["
Int 143..144 "0"
RBracket 144..145 "]"
Semi 145..146 ";"
Let 151..154 "let"
Ident 155..161 "second"
Colon 161..162 ":"
Ident 163..165 "u8"
Eq 166..167 "="
LParen 168..169 "("
Star 169..170 "*"
Ident 170..174 "data"
RParen 174..175 ")"
LBracket 175..176 "["
Int 176..177 "1"
RBracket 177..178 "]"
Semi 178..179 ";"
Return 184..190 "return"
Ident 191..196 "first"
EqEq 197..199 "=="
Int 200..204 "0x43"
AndAnd 205..207 "&&"
Ident 208..214 "second"
EqEq 215..217 "=="
Int 218..222 "0x52"
Semi 222..223 ";"
RBrace 224..225 "}"
Eof 226..226 ""
```

</details>

<details><summary>parse: actual output, exit 0</summary>

```text
0 @0:0..18 Import { path: [Name { span: Span { source: SourceId(0), start: 7, end: 10 } }, Name { span: Span { source: SourceId(0), start: 12, end: 17 } }], alias: None }
1 @0:40..45 NamedType { path: [Name { span: Span { source: SourceId(0), start: 40, end: 45 } }], arguments: [] }
2 @0:39..45 ReferenceType { mutable: false, target: NodeId(1) }
3 @0:33..45 Parameter { mutable: false, name: Name { span: Span { source: SourceId(0), start: 33, end: 37 } }, ty: NodeId(2) }
4 @0:50..54 NamedType { path: [Name { span: Span { source: SourceId(0), start: 50, end: 54 } }], arguments: [] }
5 @0:64..74 Path { components: [Name { span: Span { source: SourceId(0), start: 64, end: 69 } }, Name { span: Span { source: SourceId(0), start: 71, end: 74 } }] }
6 @0:75..79 Path { components: [Name { span: Span { source: SourceId(0), start: 75, end: 79 } }] }
7 @0:64..80 Call { callee: NodeId(5), arguments: [NodeId(6)] }
8 @0:83..84 Literal { kind: Int }
9 @0:64..84 Binary { operator: Lt, left: NodeId(7), right: NodeId(8) }
10 @0:102..107 Literal { kind: False }
11 @0:95..108 Return { value: Some(NodeId(10)) }
12 @0:85..114 Block { statements: [NodeId(11)] }
13 @0:61..114 If { condition: NodeId(9), then_block: NodeId(12), otherwise: None }
14 @0:130..132 NamedType { path: [Name { span: Span { source: SourceId(0), start: 130, end: 132 } }], arguments: [] }
15 @0:137..141 Path { components: [Name { span: Span { source: SourceId(0), start: 137, end: 141 } }] }
16 @0:136..141 Unary { operator: Star, mutable: false, operand: NodeId(15) }
17 @0:135..142 Group { value: NodeId(16) }
18 @0:143..144 Literal { kind: Int }
19 @0:135..145 Index { receiver: NodeId(17), index: NodeId(18) }
20 @0:119..146 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 123, end: 128 } }, ty: Some(NodeId(14)), value: NodeId(19) }
21 @0:163..165 NamedType { path: [Name { span: Span { source: SourceId(0), start: 163, end: 165 } }], arguments: [] }
22 @0:170..174 Path { components: [Name { span: Span { source: SourceId(0), start: 170, end: 174 } }] }
23 @0:169..174 Unary { operator: Star, mutable: false, operand: NodeId(22) }
24 @0:168..175 Group { value: NodeId(23) }
25 @0:176..177 Literal { kind: Int }
26 @0:168..178 Index { receiver: NodeId(24), index: NodeId(25) }
27 @0:151..179 Binding { mutable: false, name: Name { span: Span { source: SourceId(0), start: 155, end: 161 } }, ty: Some(NodeId(21)), value: NodeId(26) }
28 @0:191..196 Path { components: [Name { span: Span { source: SourceId(0), start: 191, end: 196 } }] }
29 @0:200..204 Literal { kind: Int }
30 @0:191..204 Binary { operator: EqEq, left: NodeId(28), right: NodeId(29) }
31 @0:208..214 Path { components: [Name { span: Span { source: SourceId(0), start: 208, end: 214 } }] }
32 @0:218..222 Literal { kind: Int }
33 @0:208..222 Binary { operator: EqEq, left: NodeId(31), right: NodeId(32) }
34 @0:191..222 Binary { operator: AndAnd, left: NodeId(30), right: NodeId(33) }
35 @0:184..223 Return { value: Some(NodeId(34)) }
36 @0:55..225 Block { statements: [NodeId(13), NodeId(20), NodeId(27), NodeId(35)] }
37 @0:20..225 Function { name: Name { span: Span { source: SourceId(0), start: 23, end: 32 } }, parameters: [NodeId(3)], result: NodeId(4), from: None, body: NodeId(36) }
38 @0:20..225 Item { public: false, declaration: NodeId(37) }
39 @0:0..226 Program { module: None, imports: [NodeId(0)], items: [NodeId(38)] }
```

</details>
