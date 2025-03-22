# 📜 Aoi Language Syntax

Aoi is a simple, interpreted programming language with a clean and easy-to-use syntax. Below is a guide to its syntax and supported features.

---

## 🔹 Variables

Variables in Aoi are dynamically typed and can be assigned using the `var` keyword.

```aoi
var x = 10;
var name = "Aoi";
```

---

## 🔹 Input/Output

### Taking User Input:

Aoi allows user input using the `scan` function.

```aoi
var x;
scan(x);
write("Hello: " + x);
```

### Printing Output:

The `write` function is used to display output.

```aoi
write("Hello, world!");
```

---

## 🔹 Conditional Statements

Aoi supports `if` and `else` for decision-making.

```aoi
var age = 18;
if (age >= 18) {
    write("You are an adult.");
} else {
    write("You are a minor.");
}
```

---

## 🔹 Loops

### `while` Loop:

```aoi
var count = 0;
while (count < 5) {
    write(count);
    count = count + 1;
}
```

### `for` Loop:

```aoi
for (var i = 0; i < 5; i = i + 1) {
    write(i);
}
```

---

## 🔹 Functions

Functions are declared using the `fun` keyword and support parameters and return values.

```aoi
fun add(a, b) {
    return a + b;
}

write(add(3, 5)); // Output: 8
```

---

## 🔹 Arithmetic Operations

Aoi supports basic arithmetic operations:

```aoi
var sum = 5 + 3;  // Addition
var diff = 10 - 4; // Subtraction
var product = 6 * 7; // Multiplication
var quotient = 20 / 4; // Division
write(" " + sum + " " + diff + " " + product + " " + quotient);
```

---

## 🔮 Upcoming Features

Aoi is evolving! Planned features include:

- **Arrays** 📦 - Support for lists and collections
- **Classes** 🏗️ - Object-oriented programming capabilities

Stay tuned for future updates! 🚀

