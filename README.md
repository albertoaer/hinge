# Hinge

The `Hinge-cli` is a very basic library intended for command line interface application development

## Example

Let's create a Hinge object to match a boolean with a flag (*foo*), an item with a flag (*bar*) and an argument preceded by no flag (*baz*).

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

We can extract the values we are interested from the `HingeOutput` object.

Here I wrote a dummy example adapted for the previous builded `Hinge`. The program will do the following:
- Try to get *bar*, then print it
- Try to get *foo*, then check whether it's true
  - if it's true, try to get *baz*, then print it

```rs
let result = hinge.apply_tokens(env::args().skip(1))?;

let bar: String = result.get_item("bar")?.clone().try_into()?;

print!("Output: {}", bar);

if result.get_item("foo")?.is_true() {
  let baz: String = result.get_item("baz")?.clone().try_into()?;
  println!(" {}", baz)
} else {
  println!("")
}
```

by providing again the same arguments we get the following result:

```
Output: hello everybody
```

## Upcoming features

- *Map* node
- More ways to define *lists*