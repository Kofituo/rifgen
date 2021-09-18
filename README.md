# rust_interface_file_generator
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![IntelliJ IDEA](https://img.shields.io/badge/IntelliJIDEA-000000.svg?style=for-the-badge&logo=intellij-idea&logoColor=white)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/rust_interface_file_generator/)

Program for translating libraries written in Rust to interface files.
It works with [flapigen](https://github.com/Dushistov/flapigen-rs). <b>For instructions on how to integrate with your project, click [here](https://docs.rs/rust_interface_file_generator/)

</b>Suppose you have the following Rust code:

```rust
struct Foo {
    data: i32
}

impl Foo {
    fn new(val: i32) -> Foo {
        Foo{data: val}
    }

    fn f(&self, a: i32, b: i32) -> i32 {
        self.data + a + b
    }

    fn set_field(&mut self, v: i32) {
        self.data = v;
    }
}
```

Using [flapigen](https://github.com/Dushistov/flapigen-rs), you'd have to write an interface file similar to
```rust
foreign_class!(class Foo {
    self_type Foo;
    constructor Foo::new(_: i32) -> Foo;
    fn Foo::set_field(&mut self, _: i32);
    fn Foo::f(&self, _: i32, _: i32) -> i32;
});
```
in order to write in Java something like this:

```Java
Foo foo = new Foo(5);
int res = foo.f(1, 2);
assert res == 8;
```
or in C++ something like this:

```C++
Foo foo(5);
int res = foo.f(1, 2);
assert(res == 8);
```

<h3>This program generates the interface file, so you can focus more time on your code

Other Features:</h3><h4>

✅ Fast and easy to use

✅ Specify style of the resulting code i.e. Whether CamelCase or snake_case

✅ Works, with `structs`, `enums`, `trait`

✅ You don't have to worry about the "order" in which code in the interface has to be

## Users Guide
<b>[Read the `rust_interface_file_gen` users guide here!](https://docs.rs/rust_interface_file_generator/)

[View on crates.io](https://crates.io/crates/rust_interface_file_generator)
