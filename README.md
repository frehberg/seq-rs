# seq module

The module _seq_ provides a generic sequence container _Seq_ for Rust.

_Seq_ is a lightweight container of data sequences (LIFO), managing dynamic list without
dynamic memory allocation involved. Sequences are stored in stack frames of function contexts.
Each element of a sequence has an individual lifetime `'a` managed by the Rust compiler.

Add the following dependency to your Cargo.toml file:
```toml
## Cargo.toml file
[dependencies]
seq = "0.3.1"
```
## Definition
_Seq_ is defined as generic enum. _Seq_ is a sequence of data of type T and the top most
element has lifetime 'a.

Either a sequence is  _Empty_ or a sequence is a construction of a new value
(head or first (ft)) on-top of another sequence (the tail or rest (rt)). The lifetime of the tail must be at least as
long as the one of the head. Two kind of constructions are possible, depending on location of tail in memory-heap or
on stack.
```rust
pub enum Seq<'a, T: 'a> {
   Empty,
   ConsRef(T, &'a Seq<'a, T>),
   ConsOwn(T, Box<Seq<'a, T>>),
}
```
Explaning the enum variants:
* `Empty`: The empty sequence `<>`
* `ConsRef(head, tail)`: Constructs a new sequence with `head` being the first element and `tail` referencing another,
  borrowed sequence. This variant permits construction of sequences using stack-allocated
  data solely.
* `ConsOwn(head, boxedtail)`: Constructs a new sequence with `head` being the first element and `boxedtail` referencing
  another, owned, boxed sequence. Here the tail is residing in heap allocated memory. This variant
  permits construction of sequences using heap-allocated dynamic data.

These variants may be combined with each other, representing a mixture of borrowed and owned elements. The memory
safety feature of Rust allows automated and correct management of lifetime of each element of the sequence.

The lifetime of each element of the sequence depends on the function-context it has been added to the top of the
sequence; _Empty_ is the element with longest lifetime (see image).

The following image illustrates the sequences `s`, `t`, `u`. The sequence `s` is a sub-sequence of `t`, and `t` 
being a sub-sequence of `u`; each one accessible in its function context only. 
![Illustration of sequence elements in stack frames](./doc/illustration.svg)

For use-cases where a sub-routine/expression shall return a temporary extended sequence, it is possible to construct new 
sequences using elements in heap-memory. In this case these heap-elements are boxed/owned.
![Illustration of sequence elements in stack frames and heap](./doc/illustration-with-heap.svg)

But, first and foremost, the container  _Seq_ is intended as lightweight, dynamic, stack-allocated, linked list for
use cases such as managing state/context while traversing tree-like data structures - without any dynamic
memory-allocation (heap) involved.

The sequence type `Seq` implements the trait `IntoIterator`, enabling the usage of Rust's iterator framework.

## Benchmarks
The data structure `Seq` implements a linked list.  It ranks between two kinds of collections, at first those 
using consecutive memory such as native arrays and `Vec`, and second those collections using non-consecutive
dynamic-memory such as `LinkedList`. 

The collection `Seq` is a recursive data structure that can be used without dynamic memory allocation
(similar to native arrays), but it permits linked lists similar to `LinkedList`. Due to the additional overhead
the collection `Seq` is around five times slower compared to memory intensive operations using native arrays.

The following benchmarks is memory-intensive and benefits from consecutive memory. 
It illustrates the overhead of `Seq` compared to native arrays and `Vec`, and the cost storing data in stack-frames.

The test performs a recursive function-call, each call adding a new element to a list (starting with 0 and incrementing each step), 
summing up all elements in the list. Finally all results are cumulated. 
The test is performed for lists with maximum length 128, and 256 and 512.

Each element of the list represents a recursive function call, and corresponding stack-frame. The additional overhead
for the data-type `Seq` in each stack-frame and the non-consecutive memory might explain the overhead of factor 12 to 20
compared to native arrays and `Vec`. Performance of `Seq` (bench_seq_) can be compared to the benchmarks of
`LinkedList` (bench_list_). The difference is that 'Seq' does not require dynamic memory allocation.

In terms of memory-footprint, each Seq-element introduces an overhead of around 16 bytes on 64bit architectures,
for the enum-discriminator and the reference to the _tail_, whereas in this test the payload of each
Seq-element is 4 bytes only (`Seq<u32>`).
```commandline
test tests::bench_array_128 ... bench:       1,111 ns/iter (+/- 28)
test tests::bench_array_256 ... bench:       4,188 ns/iter (+/- 173)
test tests::bench_array_512 ... bench:      12,576 ns/iter (+/- 258)
test tests::bench_list_128  ... bench:      14,053 ns/iter (+/- 385)
test tests::bench_list_256  ... bench:      51,068 ns/iter (+/- 757)
test tests::bench_list_512  ... bench:     187,861 ns/iter (+/- 2,688)
test tests::bench_seq_128   ... bench:      16,544 ns/iter (+/- 177)
test tests::bench_seq_256   ... bench:      63,611 ns/iter (+/- 988)
test tests::bench_seq_512   ... bench:     253,374 ns/iter (+/- 11,849)
test tests::bench_vec_128   ... bench:       1,823 ns/iter (+/- 54)
test tests::bench_vec_256   ... bench:       5,974 ns/iter (+/- 191)
test tests::bench_vec_512   ... bench:      19,545 ns/iter (+/- 3,061)
```
As a  conclusion of these benchmarks, the collection `Seq` should be used:
* for elements exceeding the overhead of 16 bytes.
* for hierarchical data, corresponding to stack-frames
* shared data-sequences, compacting data
* for local temporary lists
* for collections occupying space for exactly N elements (plus overhead), with N unknown at compile time.
* avoiding usage of heap-memory as required for `Vec` or `LinkedList`

## Example: Stack-only Allocated
A stack allocated sequence is based on the variants Seq::Empty and Seq::ConsRef only. The following lines are creating 
a sequence`<>|1|2|3` in stack frame of function `myfun()`. The sequence is the input parameter to function `myfunNested()`
that is construction a new sequence `t`, adding two more elements to sequence `s`. The elements of `s` are a sub-sequence 
of `t`.
```rust
extern crate seq;
use seq::Seq;

fn myfun() {
   // constructing the sequence <>|1|2|3
   let s0: Seq<u32> = Seq::Empty();
   let s1: Seq<u32> = Seq::ConsRef(1, &s0);
   let s3: Seq<u32> = Seq::ConsRef(2, &s1);
   let s3: Seq<u32> = Seq::ConsRef(3, &s2);
   myfunNested(&s3);
   println!("head of sequence s3 is {:?}", &s3); // printing 3
}

fn myfunNested(s: &Seq<u32>)
{
    // extend the sequence to <>|1|2|3|4|5
   let t: Seq<u32> = Seq::ConsRef(4, s);
   let t: Seq<u32> = Seq::ConsRef(5, &t); // re-using the identifier `t`
   println!("head of sequence t is {:?}", &t); // printing 5
}
```

## Example: Stack-and-Heap Allocated
A sequence can be a mixture of stack-allocated and heap-allocated data elements. The following sequence
```rust
extern crate seq;
use seq::Seq;

fn myfun() {
   // constructing the sequence <>|0|1|2|3
   let s0: Seq<u32> = Seq::Empty();                        // stack
   let s1: Seq<u32> = Seq::ConsRef(0, &s0);                // stack
   let s2: Box<Seq<u32>> = Box::new(Seq::ConsRef(1, &s1)); // heap
   let s3: Box<Seq<u32>> = Box::new(Seq::ConsOwn(2, s2));  // heap
   let s4: Seq<u32> = Seq::ConsOwn(Data(3, s3));           // stack
   println!("seq {:?}", &s4);
}
```
## Example: Pattern Matching
Pattern-matching is used to de-construct a sequence.
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
## Example: Dynamic sequence in nested/recursive function-calls
Sequences can be used to manage state in nested function calls. This code demonstrates how the iterator is used.
```rust
extern crate seq;
use seq::Seq;
use std::ops;

// Recursive, nested invocation while val<max. Each function-call adds a new seq-head and cumulates all values of the sequence (fold).
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
## Example: Dynamic sequence in nested/recursive function-calls, combined with heap-alloc. data
'Seq' permits mixture of stack allocated data and heap allocated data within a single linked list.
The following code is a variation of previous sample, just adding two heap-allocated elements onto top
of sequence finally. This code demonstrates how the iterator is used.
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
## Example: Demonstrating the macro seqdef!
The `seqdef!` macro defines a stack-allocated sequence variable using the speficied data list,
the last data item in the list will be the top most in the sequence (head). The macro can be used to
create a new sequence on top of another one (tail).

### Macro 1: Creating a seq variable `s`, the head will be `2`.
Is constructing a sequence s := <>|0|1|2; 
```rust
seqdef!(s; seq::empty() => 0, 1, 2);
```

### Macro 2: Creating a seq variable t without explicit `seq::empty()`. Seq `t` is identical to `s`.
Is constructing a sequence t := <>|0|1|2; 
```rust
seqdef!(t; 0, 1, 2);
```

### Macro 3: Creating a seq variable u, using Seq `s` of example 1 as tail, the head will be `5`.
On top of sequence s, constructing a sequence u := &s|3|4|5, the effictive sequence is u := <>|0|1|2|3|4|5; 
```rust
seqdef!(u; &s => 3, 4, 5);
```

### Macro 4: Creating a dynamic seq with MAX elements within stack frame, reading values from iterator
The previous macros are useful, but limited as the exact number of elements must be known at compile time. 
The previous macros are not able to deal with dynamic number of elements between 1-MAX, where MAX might be a large
value.

The  following macro `seqdef_try!` simplifies handling of dynamic number of values when constructing a sequence.
This macro reserves MAX elements on the stack
every time when entering the function context. The upper limit MAX is defined at compile time. 
At runtime the  sequence is constructed, reading the elements from the iterator and placing them
in the reserved stack-memory. When the function context is left, the stack-memory is released.
The macro seqdef_try! will declare the specified identifier as type Result<Seq<T>>. 
Rolling out the values from iterator into the reserved stack may fail if the iterator is empty, 
or if the amount exceeds MAX. The value of `x` must be checked after construction with `seqdef_try!`.

Note! No matter the number of elements returned by the iterator, the macro is always reserving 
stack-memory for MAX elements. If you choose too large, the stack might run out of memory.
The iterator provided to the macro may consume the underlying container-elements or clone each element.
```rust
  use std::mem;
  use std::ptr;
  
  fn dynamic_seq_rollout_on_stack() {
      const MAX: usize = 10;
      let large_list: &[i32] = &[42; MAX];
      
      // define x of type Result<Seq<i32>>, read all elements from array and place them on stack as sequence
      seqdef_try!(x, i32, MAX; empty() => large_list.iter());
      
      // handling the result, Ok or Error
      match &x {
          &Ok(ref sequence) => println!("large sum {}", (&*sequence).into_iter().fold(0i32, ops::Add::add)),
          &Err(reason) => println!("roll out failed due to {}", reason)
      }
  }
```
