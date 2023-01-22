# Operators

## Unary Prefix

Unary operations of the form `<operator><operand>`. The result type
varies on the operand type.

| Operator | Description
| -------- | -----------
| +        | Return value of operand (this essentially a no-op)
| -        | Return negated value of operand

## Boolean Prefix

Boolean prefix operators may only be applied to objects that can be
converted to `Bool`. Currently, this includes `Bool`s and `nil` only.

| Operator | Description
| !        | Boolean NOT
| !!       | Convert to `Bool`

## Binary

Binary operations of the form `LHS <operator> RHS`. The result type
varies on the operand types.

| Operator | Description
| -------- | -----------
| ^        | Power
| *        | Multiplication
| /        | Division
| //       | Floor division
| %        | Modulus
| +        | Addition
| -        | Subtraction
| .        | Attribute lookup

## Comparisons

Binary operations of the form `LHS <operator> RHS`. The result is always
a `Bool`.

| Operator | Description
| -------- | -----------
| $$       | Is/identity
| $!       | Is not
| ===      | Type-equal
| !==      | Not type-equal
| ==       | Equal
| !=       | Not equal
| <        | Less than
| <=       | Less than or equal
| >        | Greater than
| >=       | Greater than or equal

## Boolean

Binary operations of the form `LHS <operator> RHS` that may only be
applied to objects that can be converted to `Bool`. The result is always
a `Bool`. These operators are short-circuiting, so RHS will only be
evaluated if necessary.

| Operator | Description
| -------- | -----------
| &&       | AND
| \|\|     | OR

## Nil OR

| Operator | Description
| ??       | LHS if LHS is not `nil`, RHS otherwise

## In Place

Binary operations of the form `LHS <operator> RHS` where LHS is
something that can be assigned to. The result of the corresponding
binary operation is assigned to LHS.

| Operator | Description
| -------- | -----------
| *=       |
| /=       |
| +=       |
| -=       |
