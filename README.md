# 🌟 Aoi - A Rust-Based Interpreter

Aoi is a simple yet powerful interpreted programming language written in Rust. Designed for ease of use and extensibility, Aoi currently supports variables, conditionals, loops, functions, and basic input/output operations. The project is in active development, and upcoming features will make Aoi even more versatile.

---

## ✨ Features
✅ Variable declaration and assignment with dynamic typing  
✅ Conditional statements (`if`, `else`) for decision-making  
✅ Loop constructs (`for`, `while`) for iterative operations  
✅ Function support with parameters, closures, and return values  
✅ Basic input/output operations (`write`, `input`)  
✅ Memory-efficient dynamic typing using `Arc<dyn Any + Send + Sync>`  
✅ Error handling for better debugging and development  

---

## 📥 Installation
If you don't have Rust installed, install it first by running:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then, clone this repository and build the project:

```sh
git clone https://github.com/AadiS27/Interpreter.git
cd Interpreter
cargo build --release
```

---

## 🚀 Running Aoi Scripts
To run an Aoi script, follow these steps:

1️⃣ Navigate to the `src` folder:
   ```sh
   cd src
   ```
2️⃣ Write your Aoi script in `script.aoi`.
3️⃣ Execute the script using:
   ```sh
   cargo run script.aoi
   ```

---

## 📜 Example Aoi Script
Here's an example of a simple Aoi function:

```aoi
fun add(a, b) {
    return a + b;
}

write(add(3, 5)); // Output: 8
```

Aoi allows you to define reusable functions and perform mathematical operations with ease.

---

## 🔮 Roadmap & Future Enhancements
Aoi is still evolving, and several exciting features are in the pipeline:

- [ ] **Array support** 📦 — Enable the use of lists and collections
- [ ] **Class-based Object System** 🏗️ — Implement object-oriented programming capabilities
- [ ] **Improved error handling** 🛠️ — Enhance debugging and error messages
- [ ] **Expanded standard library** 📚 — Introduce more built-in functions for convenience
- [ ] **Optimized execution speed** ⚡ — Improve interpreter performance
- [ ] **More built-in operators and functions** 🔢 — Extend language expressiveness

These features will make Aoi more powerful and user-friendly. Stay tuned for future updates! 🚀

---

## 🤝 Contributing
We welcome contributions! If you have ideas or find bugs, feel free to open an issue or submit a pull request.

---

## 📜 License
This project is licensed under the MIT License. Enjoy using Aoi and help us improve it! 😊

