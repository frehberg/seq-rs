# seq module 

The _seq_ module is providing a generic sequence container _Seq_ for Rust.
 
_Seq_ is a lightweight container of data sequences, data being stacked on top of each other.

A sequence can be _Empty_ or a construction _Cons*_ of a value (called _head_ aka first _ft_) on top of another 
sequence (called _tail_ aka rest _rt_). Sequences are formed of immutable data elements; multiple sequences may share 
the same tail, permitting compact representation of hierarchical data.

Add the following dependency to your Cargo.toml file:
```toml
[dependencies]
seq = "0.1.0"
```

The sequence container _Seq_ allows to construct dynamic, linked lists; each element of the sequence is one of the 
following variants:
* Empty: The empty sequence `<>`
* ConsRef(head, tail): Constructs a new sequence with _head_ being the first element and _tail_ referencing another, borrowed sequence: `head|tail`. This variant permits construction of sequences using stack-allocated data solely". 
* ConsOwn(head, boxedtail): Constructs a new sequence with _head_ being the first element and _tail_ referencing another, owned, boxed sequence: `head|boxedtail`. Here the tail is residing in heap allocated memory. This variant permits construction of sequences using heap-allocated dynamic data.

These variants may be combined with each other, representing a mixture of borrowed and owned elements. The memory safety feature of Rust 
allows automated and correct management of lifetime of each element of the sequence.

The lifetime of each element of the sequence depends on the function context
it has been added to the top of the sequence; 'Empty' is the element with longest lifetime.

In first place _Seq_ is intended as lightweight, dynamic, stack-allocated, linked list for use cases such as traversing tree-structures.  
Traversal may be done using visitor pattern or nested functions calls; the top-most element of the sequence representing the current traversal context.
In the ideal scenarios no dynamic memory allocation shall be involved.

The sequence type `Seq` implements the trait `IntoIterator`, enabling the usage of Rust's iterator framework.
```rust
extern crate seq;
use seq::Seq;
use std::ops;

// Recursive, nested invocation while val<max. Each function-call sums up the sequence.
fn recurs(val: u32, max: u32, tail: &Seq<u32>) {
   if val < max {
      let sequence = Seq::ConsRef(val, tail);
      println!("sum is: {}", sequence.into_iter().fold(0, ops::Add::add));
      recurs(val + 1, max, &sequence);
   }
}

fn main() {
    recurs(0, 10, seq::empty());
}
```

'Seq' permits mixture of stack allocated data and heap allocated data within a single linked list.
The following code is a variation of previous sample, just adding two heap-allocated elements onto top
of sequence finally.
```rust
extern crate seq;
use seq::Seq;
use std::ops;

fn prependTwoBoxedValues <'a> (v: u32, w: u32, seq: &'a Seq<u32> ) -> Box<Seq< 'a, u32>> {
   Box::new(Seq::ConsOwn(v +1,
      Box::new(Seq::ConsRef(w, seq))))
}

// Recursive, nested invocation while val<max. Each function-call sums up the sequence.
fn recurs(val: u32, max: u32, tail: &Seq<u32>) {
   if val < max {
      let sequence = Seq::ConsRef(val, tail);
      println!("sum is: {}", sequence.into_iter().fold(0, ops::Add::add));
      recurs(val + 1, max, &sequence);
   } else {
      let sequence: Box<Seq<u32>> = prependTwoBoxedValues(max+1, max, tail);
      println!("sum is: {}", sequence.into_iter().fold(0, ops::Add::add));
   }
}

fn main() {
   recurs(0, 10, seq::empty());
}
```

_Seq_ is a sequence of data of type T and lifetime 'a.

A sequence is either
Empty '<>' or a construction Cons* of a head (ft) and tail (rt): '3|2|1|0|<>'

Either the tail is referencing borrowed data within the current execution scope (ConsRef),
or the tail is formed of owned, boxed data in heap allocated memory (ConsOwn).

A sequence can be a mixture of borrowed and owned data elements:
```rust
extern crate seq;
use seq::Seq;

fn myfun() {
   let s0: &Seq<u32> = empty();
   let s1: Seq<u32> = Seq::ConsRef(0, s0);
   let s2: Box<Seq<u32>> = Box::new(Seq::ConsRef(1, &s1));
   let s3: Box<Seq<u32>> = Box::new(Seq::ConsOwn(2, s2));
   let s4: Seq<u32> = Seq::ConsOwn(Data(3, s3));
}
```
Pattern-matching can be used to de-construct a sequence.
```rust
extern crate seq;
use seq::Seq;

fn head(sequence: &Seq<u32>) -> Option<u32> {
   match sequence {
      &Seq::Empty => Option::None,
      &Seq::ConsRef(ref ft, ref rt) => Option::Some(*ft),
      &Seq::ConsOwn(ref ft, ref rt) => Option::Some(*ft)
   }
}
```