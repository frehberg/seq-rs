#![cfg_attr(feature = "benchmark", feature(test))]
 
//! The module `seq` provides the lightweight, generic sequence container [`Seq`] for unmovable data.
//!
//! The container `Seq` is linking data of hierarchical function-scopes on top of each other,
//! forming sequences. A sequence can be embedded into the program during compile time.
//! 
//! Initially a sequence is empty. A longer sequence is constructed (see [`ConsRef`]) attaching a
//! new _head_ to the existing sequence, representing the _tail_.  The _head_ element has a shorter
//! lifetime, than all elements of the _tail_.
//! 
//! Multiple sequences may share the same _tail_, permitting memory-efficient organisation of
//! hierarchical data.
//! 
//! The associated methods [`head`] and [`tail`] have been defined for convenience reasons only.
//! The construction and deconstruction of a sequence is realized by the algebraic data-types of Rust
//! solely.
//!
//! Put this in your Cargo.toml:
//! ```toml
//! ## Cargo.toml file
//! [dependencies]
//! seq = "0.5"
//! ```
//! The "default" usage of this type as a queue is to use [`Empty`] or [`ConsRef`] to construct a
//! queue, and [`head`] and [`tail`] to deconstruct a queue into head and remaining
//! tail of a sequence.
//!
//! # Examples
//!
//! Constructing two sequences seq1 as `[1,0]` and seq2 as `[2,1,0]`, sharing data with `seq1`
//! ```rust
//! use seq::Seq;
//!
//! // constructing the sequence 'seq1'
//! const seq1: Seq<i32> = Seq::ConsRef(1, &Seq::ConsRef(0, &Seq::Empty));
//!
//! // construction the sequence 'seq2' sharing data with 'seq1'
//! const seq2: Seq<i32> = Seq::ConsRef(2, &seq1);
//! ```
//! Deconstructing a sequence
//! ```rust
//! use seq::Seq;
//!
//! fn print_head<'a>(seq: &'a Seq<i32>) {
//!    println!("head {}", seq.head().unwrap());
//! }
//! ```
//! Extend an existing sequence. Note the lifetime of the return type matches the one of the tail.
//! ```rust
//! use seq::Seq;
//!
//! fn extend<'a>(head: i32, tail: &'a Seq<i32>) -> Seq<'a, i32> {
//!    return Seq::ConsRef(head, tail);
//! }
//! ```
//! Extend an existing sequence with dynamic element residing in heap-memory
//! ```rust
//! use seq::Seq;
//!
//! fn extend_boxed<'a>(head: i32, tail: &'a Seq<i32>) -> Box<Seq<'a, i32>> {
//!    return Box::new(Seq::ConsRef(head, tail));
//! }
//! ```
//! Iterate a sequence
//! ```rust
//! use seq::Seq;
//!
//! fn sum_up(seq: &Seq<i32>) -> i32 {
//!    return seq.into_iter().fold(0, |x, y| x + y);
//! }
//! ```
//! [`Empty`]: enum.Seq.html#variant.Empty
//! [`ConsRef`]: enum.Seq.html#variant.ConsRef
//! [`tail`]:  #method.tail
//! [`head`]:  #method.head
//! [`Seq`]: enum.Seq.html

use std::fmt;
use std::iter::Iterator;


/// A single-ended, growable, unmovable queue of data, linking constant data with dynamic data.
///
/// The "default" usage of this type as a queue is to use [`Empty`] or [`ConsRef`] to construct a
/// queue, and [`head`] and [`tail`] to deconstruct a queue into head and remaining
/// tail of a sequence.
///
/// # Examples
///
/// Constructing two sequences seq1 as `[1,0]` and seq2 as `[2,1,0]`, sharing data with `seq1`
/// ```rust
/// use seq::Seq;
/// // constructing the sequence 'seq1'
/// const seq1: Seq<i32> = Seq::ConsRef(1, &Seq::ConsRef(0, &Seq::Empty));
///
/// // construction the sequence 'seq2' sharing data with 'seq1'
/// const seq2: Seq<i32> = Seq::ConsRef(2, &seq1);
/// ```
/// Deconstructing a sequence into the [`head`] and [`tail`]
/// ```rust
/// use seq::Seq;
///
/// fn deconstruct<'a>(seq: &'a Seq<i32>) {
///    let head = seq.head().unwrap();
///    let tail = seq.tail().unwrap();
///    // more code here
///    // ...
/// }
/// ```
/// Extend an existing sequence. Note the lifetime of the return type matches the one of the tail.
/// ```rust
/// use seq::Seq;
///
/// fn extend<'a>(head: i32, tail: &'a Seq<i32>) -> Seq<'a, i32> {
///    return Seq::ConsRef(head, tail);
/// }
/// ```
/// Extend an existing sequence with dynamic element residing in heap-memory
/// ```rust
/// use seq::Seq;
///
/// fn extend_boxed<'a>(head: i32, tail: &'a Seq<i32>) -> Box<Seq<'a, i32>> {
///    return Box::new(Seq::ConsRef(head, tail));
/// }
/// ```
/// Iterate a sequence
/// ```rust
/// use seq::Seq;
///
/// fn sum_up(seq: &Seq<i32>) -> i32 {
///    return seq.into_iter().fold(0, |x, y| x + y);
/// }
/// ```
/// [`Empty`]: enum.Seq.html#variant.Empty
/// [`ConsRef`]: enum.Seq.html#variant.ConsRef
/// [`tail`]:  #method.tail
/// [`head`]:  #method.head
#[derive(Clone)]
pub enum Seq<'a, T: 'a> {
    /// The empty sequence
    Empty,
    /// Constructing a sequence with head data and reference to a tail
    ConsRef(T, &'a Seq<'a, T>),
    /// Constructing a sequence with head data and reference to boxed tail
    #[cfg(not(feature = "lite-seq"))]
    ConsOwn(T, Box<Seq<'a, T>>),
}


/// Seq method implementations
impl<'a, T: 'a> Seq<'a, T> {
    /// Returns a reference to the head-element
    pub fn head(&'a self) -> Option<&'a T> {
        match self {
            &Seq::Empty => Option::None,
            &Seq::ConsRef(ref ft1, _) => Option::Some(&*ft1),
            #[cfg(not(feature = "lite-seq"))]
            &Seq::ConsOwn(ref ft1, _) => Option::Some(&*ft1),
        }
    }

    /// Returns reference to the tail
    pub fn tail(&'a self) -> Option<&'a Seq<T>> {
        match self {
            &Seq::Empty => Option::None,
            &Seq::ConsRef(_, ref rt1) => Option::Some(*rt1),
            #[cfg(not(feature = "lite-seq"))]
            &Seq::ConsOwn(_, ref rt1) => Option::Some(&**rt1),

        }
    }

    pub fn len(&'a self) -> usize {
         match self {
            &Seq::Empty => 0,
             &Seq::ConsRef(_, ref rt1) => 1 + rt1.len(),
            #[cfg(not(feature = "lite-seq"))]
             &Seq::ConsOwn(_, ref rt1) => 1 + rt1.len(),
        }
    }
}


/// The seqdef! macro defines a stack-allocated sequence variable for the speficied data list,
/// the last data item in the list will be the top most in the sequence.
///
/// Example 1) Creating a seq variable s where 2 is the top most data item
/// `seqdef!(s; empty() => 0, 1, 2);`
///
/// Example 2) Creating a seq variable t without explicit empty(). Seq is identical to `s`.
/// `seqdef!(t; 0, 1, 2);`
///
/// Example 3) Creating a seq variable u, using Seq `s` as tail of example 1.
/// `seqdef!(u; &s => 3, 4, 5);`
#[macro_export]
macro_rules! seqdef {

   ($id:ident; $($ftx:expr),* ) => {
        let $id =  $crate::Seq::Empty;
        $(
        let $id =  $crate::Seq::ConsRef( $ftx, & $id );
        )*
   };

   ($id:ident; $rt:expr => $ft:expr ) => {
        let $id =  $crate::Seq::ConsRef( $ft, $rt );
   };

   ($id:ident; $rt:expr => $ft0:expr, $($ftx:expr),* ) => {
        let $id =  $crate::Seq::ConsRef( $ft0, $rt );
        $(
        let $id =  $crate::Seq::ConsRef( $ftx, & $id );
        )*
   };
}

/// Function returns static reference to empty list
pub fn empty<T>() -> &'static Seq<'static, T> { &Seq::Empty }

/// By default a sequence is empty
impl<'a, T> Default for Seq<'a, T> {
    fn default() -> Seq<'a, T> { Seq::Empty }
}

/// Two sequences of type T are equal in case of identical length and sequence of equal data elements.
impl<'a, T: PartialEq> PartialEq for Seq<'a, T> {
    fn eq(&self, other: &Seq<'a, T>) -> bool {
        match (self, other) {
            (&Seq::Empty, &Seq::Empty) => true,
            (&Seq::ConsRef(ref ft1, ref rt1), &Seq::ConsRef(ref ft2, ref rt2))
            => ft1 == ft2 && rt1 == rt2,
            #[cfg(not(feature = "lite-seq"))]
            (&Seq::ConsRef(ref ft1, ref rt1), &Seq::ConsOwn(ref ft2, ref rt2))
            => ft1 == ft2 && *rt1 == &**rt2,
            #[cfg(not(feature = "lite-seq"))]
            (&Seq::ConsOwn(ref ft1, ref rt1), &Seq::ConsRef(ref ft2, ref rt2))
            => ft1 == ft2 && &**rt1 == *rt2,
            #[cfg(not(feature = "lite-seq"))]
            (&Seq::ConsOwn(ref ft1, ref rt1), &Seq::ConsOwn(ref ft2, ref rt2))
            => ft1 == ft2 && rt1 == rt2,
            _ => false,
        }
    }
}

/// Debug format of a sequence prints the head element only
impl<'a, T: fmt::Debug> fmt::Debug for Seq<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Seq::Empty => write!(f, "<>"),
            &Seq::ConsRef(ref ft, _) => write!(f, "<{:?},...>", ft),
            #[cfg(not(feature = "lite-seq"))]
            &Seq::ConsOwn(ref ft, _) => write!(f, "<{:?},...>", ft),
        }
    }
}

/// A sequence implements the `IntoIterator` trait
///
/// # Example
/// ```rust
/// use seq::Seq;
///
/// fn sum_up(seq: &Seq<i32>) -> i32 {
///    return seq.into_iter().fold(0, |x, y| x + y);
/// }
/// ```
impl<'a, T: 'a> IntoIterator for &'a Seq<'a, T> {
    type Item = &'a T;
    type IntoIter = SeqIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SeqIterator { cur: &self }
    }
}

/// The sequence iterator representation
pub struct SeqIterator<'a, T: 'a> {
    cur: &'a Seq<'a, T>,
}

/// The sequence iterator behavior implementation
impl<'a, T: 'a> Iterator for SeqIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            &Seq::Empty => Option::None,
            &Seq::ConsRef(ref ft, ref rt) => {
                self.cur = &*rt;
                Option::Some(&*ft)
            }
            #[cfg(not(feature = "lite-seq"))]
            &Seq::ConsOwn(ref ft, ref rt) => {
                self.cur = &**rt; // deref boxed rest
                Option::Some(&*ft)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Seq;
    #[cfg(not(feature = "lite-seq"))]
    use super::SeqIterator;
    use super::empty;
    #[cfg(not(feature = "lite-seq"))]
    use std::ops;

    struct MyData(&'static str);

    // this static ring has 4 elements only
    static CYC_A: Seq<MyData> = Seq::ConsRef(MyData("Forever"), &CYC_D); // len()==7
    static CYC_B : Seq<MyData> = Seq::ConsRef(MyData("Round"), &CYC_A); // len()==5
    static CYC_C : Seq<MyData> = Seq::ConsRef(MyData("And"), &CYC_B); // len()==3
    static CYC_D : Seq<MyData> = Seq::ConsRef(MyData("Round"), &CYC_C); // len()==5

    #[test]
    fn test_cyclic() {
        // take first 12 elements from cyclic ring and count the characters
        let sum = CYC_A.into_iter().take(3*4).fold(0, | x, y| x + y.0.len());
        assert_eq!(3*20, sum);
    }

    #[test]
    fn test_consref() {
        let s = Seq::ConsRef(1, &Seq::ConsRef(0, &Seq::Empty));
        assert_ne!(&s, empty());
    }

    fn recurs(val: u32, max: u32, base: &Seq<u32>) {
        let ext = Seq::ConsRef(val, base);

        if val < max {
            recurs(val + 1, max, &ext);
        }
    }

    #[test]
    fn test_empty() {
        let s0: &Seq<u32> = empty();
        let s1 = Seq::ConsRef(1u32, s0);
        assert_eq!(s0.len(), 0);
        assert_eq!(s1.len(), 1);

        assert_eq!(s0, empty());
        assert_ne!(&s1, empty());
    }

    #[cfg(not(feature = "lite-seq"))]
    #[test]
    fn test_shared() {
        let s0: &Seq<u32> = empty();

        // shared sequence with head s2: <2,1>
        let s1 = Seq::ConsRef(1u32, s0);
        let s2 = Seq::ConsRef(2u32, &s1);

        // t-branch prepending elements to s2: <4,3,2,1>
        let t3 = Seq::ConsRef(3u32, &s2);
        let t4 = Seq::ConsRef(4u32, &t3);

        // r-branch prepending elements to s2: <4,3,2,1>
        let r3 = Seq::ConsRef(3u32, &s2);
        let r4 = Seq::ConsRef(4u32, &r3);

        // z-branch prepending elements to s2: <33,2,1>
        let z = Seq::ConsRef(33u32, &s2);
        assert_eq!(s0.len(), 0);
        assert_eq!(s1.len(), 1);
        assert_eq!(s2.len(), 2);

        assert_eq!(s0, empty());
        assert_eq!(s1, s1);
        assert_eq!(s2, s2);
        assert_eq!(t3, r3);
        assert_eq!(t4, r4);

        // z-branch must not equal to t3 or t4
        assert_ne!(z, t3);
        assert_ne!(z, t4);
    }

    #[test]
    fn test_match() {
        let s0: &Seq<u32> = empty();
        let s1 = Seq::ConsRef(1u32, s0);
        let s2 = Seq::ConsRef(2u32, &s1);

        match &s2 {
            &Seq::Empty => assert!(false, "seq was not empty!"),
            &Seq::ConsRef(h, ref tail) => {
                let t: &Seq<u32> = &*tail;

                assert_eq!(h, 2u32);

                match t {
                    &Seq::Empty => assert!(false, "seq was not empty!"),
                    &Seq::ConsRef(h2, _) => {
                        assert_eq!(h2, 1u32);
                    }
                     #[cfg(not(feature = "lite-seq"))]
                    _ => assert!(false, "seq was not owned!"),
                }
            }
            #[cfg(not(feature = "lite-seq"))]
            _ => assert!(false, "seq was not owned!"),
        }

        println!("seq: {:?}", &s2);
    }

    #[test]
    fn test_printformat() {
        let s0: &Seq<u32> = empty();
        let s1 = Seq::ConsRef(1u32, s0);

        println!("seq: {:?}, {:?}", s0, &s1);
    }

    #[test]
    fn test_recursion() {
        recurs(0, 9, empty());
    }

    #[cfg(not(feature = "lite-seq"))]
    fn prepend_boxed<'a>(start: u32, seq: &'a Seq<u32>) -> Box<Seq<'a, u32>> {
        Box::new(
            Seq::ConsOwn(
                start + 3,
                Box::new(
                    Seq::ConsOwn(
                        start + 2,
                        Box::new(
                            Seq::ConsOwn(
                                start + 1,
                                Box::new(
                                    Seq::ConsRef(
                                        start,
                                        seq))))))))
    }

    #[cfg(not(feature = "lite-seq"))]
    #[test]
    fn test_box() {
        let s0: &Seq<u32> = empty();
        let s1: Box<Seq<u32>> = prepend_boxed(1, s0);

        assert_eq!(s0, empty());
        assert_ne!(&*s1, empty());
    }

    #[derive(PartialEq, PartialOrd, Debug)]
    struct Data([u32; 8]);

    #[cfg(not(feature = "lite-seq"))]
    #[test]
    fn test_box_struct() {
        let s0: &Seq<Data> = empty();
        let s1: Seq<Data> = Seq::ConsRef(Data([0; 8]), s0);
        let s2: Box<Seq<Data>> = Box::new(Seq::ConsRef(Data([1; 8]), &s1));
        let s3: Box<Seq<Data>> = Box::new(Seq::ConsOwn(Data([2; 8]), s2));
        let s4: Seq<Data> = Seq::ConsOwn(Data([3; 8]), s3);


        assert_eq!(s4.len(), 4);

        assert_eq!(&s4, &s4);
    }

    #[cfg(not(feature = "lite-seq"))]
    #[test]
    fn test_iter() {
        let s0: &Seq<u32> = empty();
        let s1 = Seq::ConsRef(1u32, s0);
        let s2 = Seq::ConsRef(2u32, &s1);
        let s3 = Seq::ConsRef(3u32, &s2);
        let s4 = Seq::ConsRef(4u32, &s3);
        let iter: SeqIterator<u32> = s4.into_iter();
        let sum = iter.fold(0, ops::Add::add);

        assert_eq!(sum, 10);
    }

    #[cfg(not(feature = "lite-seq"))]
    #[test]
    fn test_iter_boxed() {
        let seq: Box<Seq<u32>> = prepend_boxed(1, empty());

        let iter: SeqIterator<u32> = seq.into_iter();
        let sum = iter.fold(0, ops::Add::add);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_macro() {
        seqdef!(s; empty() => 0);
        assert_ne!(&s, empty());

        seqdef!(t; &s => 1, 2, 3);
        assert_ne!(&t, empty());

        seqdef!(u; empty() => 0, 1, 2, 3);
        assert_ne!(&u, empty());

        assert_eq!(&u, &t);

        seqdef!(v; 0);
        assert_eq!(&v, &s);

        seqdef!(w; 0, 1, 2, 3);
        assert_eq!(&w, &u);
    }

    #[test]
    fn test_head_tail() {
        let s: &Seq<u32> = empty();
        let s = Seq::ConsRef(1u32, s);
        let s = Seq::ConsRef(2u32, &s);
        let s = Seq::ConsRef(3u32, &s);

        let ft = s.head();
        let rt = s.tail();

        assert_eq!(ft.unwrap(), &3);
        assert_eq!(rt.unwrap().head().unwrap(), &2);
    }
}


#[cfg(all(feature = "benchmark", test))]
mod benchmark {
    extern crate test;

    use super::Seq;
    use super::empty;
    use std::ops;
    use std::vec;
    use std::collections::LinkedList;

    // Returns cumulation of  0, 0+1, 0+1+2, 0+1+2+3, ... 0+1+2+..+(N-1)
    fn sum_of_sums(n: u32) -> u32
    {
        let cumulated = (n * (n + 1) * ((2 * n) + 1) / 6) + ((n * (n + 1)) / 2);
        cumulated / 2
    }

    // Recursive function, adding an element and cumulate the sums, until N-1 is reached.
    fn recurs_stack_list(l: &mut LinkedList<u32>, cnt: u32, n: u32) -> u32 {
        if cnt < n {
            l.push_back(cnt);
            let sum = l.iter().fold(0u32, ops::Add::add);
            let r = sum + recurs_stack_list(l, cnt + 1, n);
            l.pop_back();
            r
        } else {
            0
        }
    }

    #[bench]
    fn bench_list_008(b: &mut test::Bencher) {
        const N: u32 = 8;

        b.iter(|| {
            let mut l = LinkedList::new();

            let sum = recurs_stack_list(&mut l, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_list_016(b: &mut test::Bencher) {
        const N: u32 = 16;

        b.iter(|| {
            let mut l = LinkedList::new();

            let sum = recurs_stack_list(&mut l, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_list_032(b: &mut test::Bencher) {
        const N: u32 = 32;

        b.iter(|| {
            let mut l = LinkedList::new();

            let sum = recurs_stack_list(&mut l, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_list_064(b: &mut test::Bencher) {
        const N: u32 = 64;

        b.iter(|| {
            let mut l = LinkedList::new();

            let sum = recurs_stack_list(&mut l, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_list_128(b: &mut test::Bencher) {
        const N: u32 = 128;

        b.iter(|| {
            let mut l = LinkedList::new();

            let sum = recurs_stack_list(&mut l, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_list_256(b: &mut test::Bencher) {
        const N: u32 = 256;

        b.iter(|| {
            let mut l = LinkedList::new();

            let sum = recurs_stack_list(&mut l, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_list_512(b: &mut test::Bencher) {
        const N: u32 = 512;

        b.iter(|| {
            let mut l = LinkedList::new();

            let sum = recurs_stack_list(&mut l, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    // Recursive function, adding an element and cumulate the sums, until N-1 is reached.
    fn recurs_stack_vec(v: &mut vec::Vec<u32>, cnt: u32, n: u32) -> u32 {
        if cnt < n {
            v.push(cnt);
            let sum = v.iter().fold(0u32, ops::Add::add);
            let r = sum + recurs_stack_vec(v, cnt + 1, n);
            v.truncate(cnt as usize);
            r
        } else {
            0
        }
    }

    #[bench]
    fn bench_vec_008(b: &mut test::Bencher) {
        const N: u32 = 8;

        b.iter(|| {
            let mut v = vec::Vec::new();
            let sum = recurs_stack_vec(&mut v, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_vec_016(b: &mut test::Bencher) {
        const N: u32 = 16;

        b.iter(|| {
            let mut v = vec::Vec::new();
            let sum = recurs_stack_vec(&mut v, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_vec_032(b: &mut test::Bencher) {
        const N: u32 = 32;

        b.iter(|| {
            let mut v = vec::Vec::new();
            let sum = recurs_stack_vec(&mut v, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_vec_064(b: &mut test::Bencher) {
        const N: u32 = 64;

        b.iter(|| {
            let mut v = vec::Vec::new();
            let sum = recurs_stack_vec(&mut v, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_vec_128(b: &mut test::Bencher) {
        const N: u32 = 128;

        b.iter(|| {
            let mut v = vec::Vec::new();
            let sum = recurs_stack_vec(&mut v, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_vec_256(b: &mut test::Bencher) {
        const N: u32 = 256;

        b.iter(|| {
            let mut v = vec::Vec::new();
            let sum = recurs_stack_vec(&mut v, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_vec_512(b: &mut test::Bencher) {
        const N: u32 = 512;

        b.iter(|| {
            let mut v = vec::Vec::new();
            let sum = recurs_stack_vec(&mut v, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    // Recursive function, adding an element and cumulate the sums, until N-1 is reached.
    fn recurs_stack_seq(s: &Seq<u32>, cnt: u32, n: u32) -> u32 {
        if cnt < n {
            let ext_s = Seq::ConsRef(cnt, s);

            let sum = ext_s.into_iter().fold(0u32, ops::Add::add);
            sum + recurs_stack_seq(&ext_s, cnt + 1, n)
        } else {
            0
        }
    }

    #[bench]
    fn bench_seq_008(b: &mut test::Bencher) {
        const N: u32 = 8;

        b.iter(|| {
            let sum = recurs_stack_seq(empty(), 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_seq_016(b: &mut test::Bencher) {
        const N: u32 = 16;

        b.iter(|| {
            let sum = recurs_stack_seq(empty(), 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_seq_032(b: &mut test::Bencher) {
        const N: u32 = 32;

        b.iter(|| {
            let sum = recurs_stack_seq(empty(), 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_seq_064(b: &mut test::Bencher) {
        const N: u32 = 64;

        b.iter(|| {
            let sum = recurs_stack_seq(empty(), 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_seq_128(b: &mut test::Bencher) {
        const N: u32 = 128;

        b.iter(|| {
            let sum = recurs_stack_seq(empty(), 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }


    #[bench]
    fn bench_seq_256(b: &mut test::Bencher) {
        const N: u32 = 256;

        b.iter(|| {
            let sum = recurs_stack_seq(empty(), 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_seq_512(b: &mut test::Bencher) {
        const N: u32 = 512;

        b.iter(|| {
            let sum = recurs_stack_seq(empty(), 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    // Recursive function, adding an element and cumulate the sums, until N-1 is reached.
    fn recurs_stack_array(a: &mut [u32], cnt: u32, n: u32) -> u32 {
        if cnt < n {
            a[cnt as usize] = cnt;

            let sum = a[..(cnt + 1) as usize].into_iter().fold(0u32, ops::Add::add);
            sum + recurs_stack_array(a, cnt + 1, n)
        } else {
            0
        }
    }

    #[bench]
    fn bench_array_008(b: &mut test::Bencher) {
        const N: u32 = 8;
        b.iter(|| {
            let mut a: [u32; N as usize] = [0; N as usize];

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_array_016(b: &mut test::Bencher) {
        const N: u32 = 16;
        b.iter(|| {
            let mut a: [u32; N as usize] = [0; N as usize];

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_array_032(b: &mut test::Bencher) {
        const N: u32 = 32;
        b.iter(|| {
            let mut a: [u32; N as usize] = [0; N as usize];

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_array_064(b: &mut test::Bencher) {
        const N: u32 = 64;
        b.iter(|| {
            let mut a: [u32; N as usize] = [0; N as usize];

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_array_128(b: &mut test::Bencher) {
        const N: u32 = 128;
        b.iter(|| {
            let mut a: [u32; N as usize] = [0; N as usize];

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_array_256(b: &mut test::Bencher) {
        const N: u32 = 256;
        b.iter(|| {
            let mut a: [u32; N as usize] = [0; N as usize];

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_array_512(b: &mut test::Bencher) {
        const N: u32 = 512;
        b.iter(|| {
            let mut a: [u32; N as usize] = [0; N as usize];

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N - 1));
            sum
        });
    }

    #[bench]
    fn bench_uninit_008(b: &mut test::Bencher) {
        const N: u32 = 8;
        b.iter(|| unsafe {
            let mut a: [u32; N as usize] = ::std::mem::uninitialized();

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N-1));
            sum
        });
    }


    #[bench]
    fn bench_uninit_016(b: &mut test::Bencher) {
        const N: u32 = 16;
        b.iter(|| unsafe {
            let mut a: [u32; N as usize] = ::std::mem::uninitialized();

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N-1));
            sum
        });
    }


    #[bench]
    fn bench_uninit_032(b: &mut test::Bencher) {
        const N: u32 = 32;
        b.iter(|| unsafe {
            let mut a: [u32; N as usize] = ::std::mem::uninitialized();

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N-1));
            sum
        });
    }


    #[bench]
    fn bench_uninit_064(b: &mut test::Bencher) {
        const N: u32 = 64;
        b.iter(|| unsafe {
            let mut a: [u32; N as usize] = ::std::mem::uninitialized();

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N-1));
            sum
        });
    }


    #[bench]
    fn bench_uninit_128(b: &mut test::Bencher) {
        const N: u32 = 128;
        b.iter(|| unsafe {
            let mut a: [u32; N as usize] = ::std::mem::uninitialized();

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N-1));
            sum
        });
    }


    #[bench]
    fn bench_uninit_512(b: &mut test::Bencher) {
        const N: u32 = 512;
        b.iter(|| unsafe {
            let mut a: [u32; N as usize] = ::std::mem::uninitialized();

            let sum = recurs_stack_array(&mut a, 0, N);
            assert_eq!(sum, sum_of_sums(N-1));
            sum
        });
    }



}
