# Hinge

The `Hinge-cli` is a very basic library intended for command line interface application developing

## Example

Let's create a Hinge object to match a boolean with a flag (**foo**), a item with a flag (**bar**) and a argument preceded by no flag (**baz**).

```rs
let hinge: Hinge = HingeBuilder::new()
  .bool_flag("foo", ('f', "foo"))
  .item_flag("bar", ('b', "bar"), false)
  .arg("baz", false)
  .build();
```

Currently there is no shortcut to apply `Hinge` to the environment arguments, however it can be evaluated through this iterator that removes the first element (the program path)

```rs
println!("{:?}", hinge.apply_tokens(env::args().skip(1)).unwrap());
```

The result shows the following for this execution: *cargo run -- --foo -b hello everybody*

```
Map({"bar": Value("hello"), "baz": Value("everybody"), "foo": True})
```