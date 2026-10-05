# Invalid source demonstrations

Actual debug-inspector runs on Linux, 2026-10-05. Each process terminated normally with exit code 1, not a signal or panic. JSON includes primary byte ranges. Paths are supplied by the caller, not included in JSON.

## unterminated-string

```cretes
fn f() -> () { let x = "oops; }
```

Exit: 1

```json
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":23,"end":31,"code":"L004","severity":"error","message":"invalid or unterminated literal","help":"close the delimiter; use supported escapes and exactly one scalar for a character","secondary":[],"notes":[]}
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":23,"end":31,"code":"P001","severity":"error","message":"expected expression, found Invalid","help":"supply a literal, name, unary operand or parenthesized expression","secondary":[],"notes":[]}
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":31,"end":31,"code":"P001","severity":"error","message":"expected RBrace, found Eof","help":"insert the expected token or correct the preceding construct","secondary":[{"source_id":0,"start":13,"end":14,"message":"opening delimiter"}],"notes":[]}
```

## missing-delimiter

```cretes
fn f() -> () { return;
```

Exit: 1

```json
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":22,"end":22,"code":"P001","severity":"error","message":"expected RBrace, found Eof","help":"insert the expected token or correct the preceding construct","secondary":[{"source_id":0,"start":13,"end":14,"message":"opening delimiter"}],"notes":[]}
```

## malformed-function

```cretes
fn () -> () {}
```

Exit: 1

```json
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":3,"end":4,"code":"P001","severity":"error","message":"expected identifier","help":"use an ASCII identifier that is not a keyword or wildcard","secondary":[],"notes":[]}
```

## invalid-number

```cretes
fn f() -> () { let x = 0b102; }
```

Exit: 1

```json
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":23,"end":28,"code":"L003","severity":"error","message":"malformed numeric literal","help":"check radix digits, separators and exponent digits; suffixes are not supported","secondary":[],"notes":[]}
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":23,"end":28,"code":"P001","severity":"error","message":"expected expression, found Invalid","help":"supply a literal, name, unary operand or parenthesized expression","secondary":[],"notes":[]}
```

## unexpected-token

```cretes
fn f() -> () { @; }
```

Exit: 1

```json
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":15,"end":16,"code":"L002","severity":"error","message":"unrecognized source character","help":"identifiers use ASCII letters, digits and underscore; check punctuation","secondary":[],"notes":[]}
{"schema_version":1,"offset_unit":"utf8-byte","range":"half-open","source_id":0,"start":15,"end":16,"code":"P001","severity":"error","message":"expected expression, found Invalid","help":"supply a literal, name, unary operand or parenthesized expression","secondary":[],"notes":[]}
```
