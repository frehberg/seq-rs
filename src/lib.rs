#![allow(unused_imports)]

use std::fmt;
use std::ops;
use std::iter::Iterator;

/// The module seq is a lightweight container of data sequences, stacked on top of each other.
///
/// A sequence can be Empty or a construction Cons of a value (Head) and a reference to the rest (Tail).
/// Sequences are formed of immutable data elements, multiple sequences may share the same Tail, permitting
/// compact representation of hierarchical data.
///
/// The sequence container allows to manage dynamic, linked lists of borrowed data without any
/// dynamic memory allocation involved (ConsRef), but permits - thanks to the memory safety feature of Rust -
/// also mixture with dynamic, boxed, owned data on memory heap (ConsOwn) if required. Each element
/// of the sequence may be associated with a different lifetime, depending on the function context
/// it has been added to the top of the sequence; 'Empty' is the element with longest lifetime.
///
/// In first place 'Seq' is intended as lightweight, dynamic, linked list for deeply, stacked,
/// nested function calls with the requirement to manage its context-state of immutable data. In
/// ideal scenarios without any dynamic memory allocation being involved.
/// The sequence 'Seq' implements the trait 'IntoIterator', enabling the usage of Rust's iterator framework.
/// ```
/// use seq;
///
/// // Recursive, nested invocation while val<max. Each function-call sums up the sequence.
/// fn recurs(val: u32, max: u32, tail: &Seq<u32>) {
///    if val < max {
///       let sequence = Seq::ConsRef(val, tail);
///       let sum = sequence.into_iter().fold(0, ops::Add::add);
///       recurs(val + 1, max, &sequence);
///    }
/// }
///
/// fn main() {
///    recurs(0, 10, seq::empty());
/// }
/// ```
/// 'Seq' permits mixture of stack allocated data and heap allocated data within a single linked list.
/// The following code is a variation of previous sample, just adding two heap-allocated elements onto top
/// of sequence finally.
/// ```
/// use seq;
///
/// fn prependTwoBoxedValues < 'a > (v: u32, w: u32, seq: & 'a Seq < u32 > ) -> Box < Seq < 'a, u32 > > {
///    Box::new(Seq::ConsOwn(v +1,
///       Box::new(Seq::ConsRef(w, seq))))
/// }
///
/// // Recursive, nested invocation while val<max. Each function-call sums up the sequence.
/// fn recurs(val: u32, max: u32, tail: &Seq<u32>) {
///    if val < max {
///       let sequence = Seq::ConsRef(val, tail);
///       println!("sum is: {}", sequence.into_iter().fold(0, ops::Add::add));
///       recurs(val + 1, max, &sequence);
///    } else {
///       let sequence: Box<Seq<u32>> = prependTwoBoxedValues(max+1, max, tail);
///       println!("sum is: {}", sequence.into_iter().fold(0, ops::Add::add));
///    }
/// }
///
/// fn main() {
///    recurs(0, 10, seq::empty());
/// }
/// ```

/// 'Seq' is a sequence of data of type T and lifetime 'a.
///
/// A sequence is either
/// Empty '<>' or a construction Cons* of a head (ft) and tail (rt): '3|2|1|0|<>'
///
/// Either the tail is referencing borrowed data within the current execution scope (ConsRef),
/// or the tail is formed of owned, boxed data in heap allocated memory (ConsOwn).
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
}
