/// The module _seq_ provides a generic sequence container _Seq_ for Rust.
///
/// _Seq_ is a generic, lightweight sequence container (LIFO), intended for the context management
/// in nested function calls using stack-allocated memory.
///
/// You can use Seq in your project adding the following dependency to your Cargo.toml file:
/// ```cargo
/// ## Cargo.toml file
/// ...
/// [dependencies]
/// seq = "0.2.0"
/// ```

use std::fmt;
use std::iter::Iterator;

/// The data type Seq is a generic data container providing linked list functionality.
/// _Seq_ is a sequence of data of type T and lifetime 'a.
///
/// Either a sequence is  _Empty_ or a sequence is a construction of a new value
/// (head or first (ft)) on-top of another sequence (the tail or rest (rt)). The lifetime of the tail must be at least as
/// long as the one of the head. Two kind of constructions are possible, depending on location of tail in memory-heap or
/// on stack.
/// ```rust
/// pub enum Seq<'a, T: 'a> {
///     Empty,
///     ConsRef(T, &'a Seq<'a, T>),
///     ConsOwn(T, Box<Seq<'a, T>>),
/// }
/// ```
/// The semantic of these variants is as follows:
/// * `Empty`: The empty sequence `<>`
/// * `ConsRef(head, tail)`: Constructs a new sequence with `head` being the first element and `tail` referencing another,
/// borrowed sequence. This variant permits construction of sequences using stack-allocated
/// data solely.
/// * `ConsOwn(head, boxedtail)`: Constructs a new sequence with `head` being the first element and `boxedtail` referencing
/// another, owned, boxed sequence. Here the tail is residing in heap allocated memory. This variant
/// permits construction of sequences using heap-allocated dynamic data.
///
/// These variants may be combined with each other, representing a mixture of borrowed and owned elements. The memory
/// safety feature of Rust allows automated and correct management of lifetime of each element of the sequence.
///
/// The lifetime of each element of the sequence depends on the function-context it has been added to the top of the
/// sequence; _Empty_ is the element with longest lifetime.
///
/// In first place, the container  _Seq_ is intended as lightweight, dynamic, stack-allocated, linked list for
/// use cases such as traversing tree-structures without any dynamic memory-allocation (heap) involved.
///
/// The sequence type `Seq` implements the trait `IntoIterator`, enabling the usage of Rust's iterator framework.
///
/// A sequence can be a mixture of borrowed and owned data elements:
/// ```
/// let s0: &Seq<u32> = empty();
/// let s1: Seq<u32> = Seq::ConsRef(0, s0);
/// let s2: Box<Seq<u32>> = Box::new(Seq::ConsRef(1, &s1));
/// let s3: Box<Seq<u32>> = Box::new(Seq::ConsOwn(2, s2));
/// let s4: Seq<u32> = Seq::ConsOwn(Data(3, s3);
/// ```
/// Pattern-matching can be used to de-construct a sequence.
/// ```
/// fn head(seq: &Seq<u32>) -> Option<u32> {
///    match self.cur {
///       &Seq::Empty => Option::None,
///       &Seq::ConsRef(ref ft, ref rt) => Option::Some(*ft),
///       &Seq::ConsOwn(ref ft, ref rt) => Option::Some(*ft)
///  }
/// ```
pub enum Seq<'a, T: 'a> {
    Empty,
    ConsRef(T, &'a Seq<'a, T>),
    ConsOwn(T, Box<Seq<'a, T>>),
}

/// The seqdef! macro defines a stack-allocated sequence variable for the speficied data list,
/// the last data item in the list will be the top most in the sequence.
///
/// Example 1) Creating a seq variable s where 2 is the top most data item
/// `seqdef!(s; seq::empty() => 0, 1, 2);`
///
/// Example 2) Creating a seq variable t without explicit seq::empty(). Seq is identical to `s`.
/// `seqdef!(t; 0, 1, 2);`
///
/// Example 3) Creating a seq variable u, using Seq `s` as tail of example 1.
/// `seqdef!(u; &s => 3, 4, 5);`
#[macro_export]
macro_rules! seqdef {
  ($id:ident; $ft:expr ) => {
        let $id =  $crate::Seq::ConsRef( $ft, $crate::empty() );
   };

   ($id:ident; $ft0:expr, $($ftn:expr),* ) => {
        let $id =  $crate::Seq::ConsRef( $ft0, $crate::empty() );
        $(
        let $id =  $crate::Seq::ConsRef( $ftn, & $id );
        )*
   };

   ($id:ident; $rt:expr => $ft:expr ) => {
        let $id =  $crate::Seq::ConsRef( $ft, $rt );
   };

   ($id:ident; $rt:expr => $ft0:expr, $($ftn:expr),* ) => {
        let $id =  $crate::Seq::ConsRef( $ft0, $rt );
        $(
        let $id =  $crate::Seq::ConsRef( $ftn, & $id );
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
            (&Seq::ConsRef(ref ft1, ref rt1), &Seq::ConsOwn(ref ft2, ref rt2))
                => ft1 == ft2 && *rt1 == &**rt2,
            (&Seq::ConsOwn(ref ft1, ref rt1), &Seq::ConsRef(ref ft2, ref rt2))
                => ft1 == ft2 && &**rt1 == *rt2,
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
            &Seq::ConsOwn(ref ft, _) => write!(f, "<{:?},...>", ft),
        }
    }
}

/// A sequence can be processed using an iterator:
/// ```
/// let sequence = empty<u32>();
/// let sum = sequence.into_iter().fold(0, ops::Add::add);
/// ```
impl<'a, T: 'a> IntoIterator for &'a Seq<'a, T> {
    type Item = &'a T;
    type IntoIter = SeqIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SeqIterator{cur: &self}
    }
}

/// The sequence iterator representation
pub struct SeqIterator<'a, T:'a> {
    cur: &'a Seq<'a, T>,
}

/// The sequence iterator behavior implementation
impl<'a, T:'a> Iterator for SeqIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            &Seq::Empty => Option::None,
            &Seq::ConsRef(ref ft, ref rt) => {
                self.cur = &*rt;
                Option::Some(&*ft)
            }
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
    use super::SeqIterator;
    use super::empty;
    use std::ops;


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

        assert_eq!(s0, empty());
        assert_ne!(&s1, empty());
    }

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
                    _ => assert!(false, "seq was not owned!"),
                }
            }
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

    fn prepend_boxed<'a>(start: u32, seq: &'a Seq<u32>) -> Box<Seq<'a, u32>> {
        Box::new(
            Seq::ConsOwn(
                start+3,
                Box::new(
                    Seq::ConsOwn(
                        start+2,
                        Box::new(
                            Seq::ConsOwn(
                                start+1,
                                Box::new(
                                    Seq::ConsRef(
                                        start,
                                        seq))))))))
    }

    #[test]
    fn test_box() {
        let s0: &Seq<u32> = empty();
        let s1: Box<Seq<u32>> = prepend_boxed(1, s0);

        assert_eq!(s0, empty());
        assert_ne!(&*s1, empty());
    }

    #[derive(PartialEq, PartialOrd, Debug)]
    struct Data ([u32; 8]);

    #[test]
    fn test_box_struct() {
        let s0: &Seq<Data> = empty();
        let s1: Seq<Data> = Seq::ConsRef(Data([0; 8]), s0);
        let s2: Box<Seq<Data>> = Box::new(Seq::ConsRef(Data([1; 8]), &s1));
        let s3: Box<Seq<Data>> = Box::new(Seq::ConsOwn(Data([2; 8]), s2));
        let s4: Seq<Data> = Seq::ConsOwn(Data([3; 8]), s3);

        assert_eq!(&s4, &s4);
    }
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

    #[test]
    fn test_iter_boxed() {
        let seq: Box<Seq<u32>> = prepend_boxed(1,empty());

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
}
