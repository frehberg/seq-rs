/// The module _seq_ provides a generic sequence container _Seq_ for Rust.
///
/// _Seq_ is a lightweight container of data sequences (LIFO), managing dynamic list without
/// dynamic memory allocation involved. Sequences are stored in stack frames of function contexts.
/// Each element of a sequence has an individual lifetime `'a` managed by the Rust compiler.
///
/// You can use Seq in your project adding the following dependency to your Cargo.toml file:
/// ```toml
/// ## Cargo.toml file
/// [dependencies]
/// seq = "0.3.0"
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
#[derive(Clone)]
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

/// The macro `seqdef_try!` places values from iterator as sequence on stack.
///
/// This macro reserves MAX elements on the stack
/// every time when entering the function context. The upper limit MAX is defined at compile time.
/// At runtime the  sequence is constructed, reading the elements from the iterator and placing them
/// in the reserved stack-memory. When the function context is left, the stack-memory is released.
/// The macro seqdef_try! will declare the specified identifier as type Result<Seq<T>>.
/// Rolling out the values from iterator into the reserved stack may fail if the iterator is empty,
/// or if the amount exceeds MAX. The value of `x` must be checked after construction with `seqdef_try!`.
///
/// Note! No matter the number of elements returned by the iterator, the macro is always reserving
/// stack-memory for MAX elements. If you choose too large, the stack might run out of memory.
/// The iterator provided to the macro may consume the underlying container-elements or clone each element.
///
/// Example
//```rust
//use std::mem;
/// use std::ptr;
///
/// fn large_seq_rollout_on_stack() {
///     const MAX: usize = 2000;
///     let large_list: &[i32] = &[42; MAX];
///     // define x of type Result<Seq<i32>>, read all elements from array and place them on stack as sequence
///     seqdef_try!(x, i32, MAX; empty() => large_list.iter());
///     // handling the result, Ok or Error
///     match &x {
///         &Ok(ref sequence) => println!("large sum {}", (&*sequence).into_iter().fold(0i32, ops::Add::add)),
///         &Err(reason) => println!("roll out failed due to {}", reason)
///     }
/// }
/// ```
#[macro_export]
macro_rules! seqdef_try {
  ($id:ident, $typ:ty, $max:expr; $rt:expr => $iterator:expr ) => {
        // array of size max-1, as the last Seq element will be outside of array
        let mut _a: [Seq<$typ>; ($max)-1 ];
        // sadly _a = Default::default() is working only array with 32 elements, the moment writing
        // therefor using this unsafe method for now
        unsafe {
            _a = mem::uninitialized();

            // DANGER ZONE: if anything panics or otherwise
            // incorrectly reads the array here, we will have
            // Undefined Behavior.

            // It's ok to mutably iterate the data, since this
            // doesn't involve reading it at all.
            // (ptr and len are statically known for arrays)
            for elem in &mut _a[..] {
                // *elem = Seq::Empty would try to drop the
                // uninitialized memory at `elem` -- bad!
                //
                // Seq::Empty doesn't allocate or do really
                // anything. It's only safe to call here
                // because we know it won't panic.
                ptr::write(elem, $crate::Seq::Empty );
            }

            // SAFE ZONE: everything is initialized.
        }


        let $id: Result<Seq<$typ>, &str> = {
            let nitems = $iterator.len();
            let mut top: Result<Seq<$typ>, &str> = Result::Err("list must not be empty");

            if nitems>$max {
                top = Result::Err("list exceeding max number");
            }
            else {
            for (i, item) in $iterator.enumerate() {
                // special case: first and only element in list => define top
                if i==0 && nitems==1 {
                    top = Result::Ok($crate::Seq::ConsRef( *item, $rt ));
                }
                // last element in list => define top
                else if i==nitems-1 {
                    unsafe { // using raw-ptr as assignment to borrowed array is not possible
                        let pre: *const Seq<$typ> = &_a[i-1];
                        top = Result::Ok($crate::Seq::ConsRef( *item, &*pre ));
                    }
                }
                // first element in list, being placed in first slot of array
                else if i==0 {
                    _a[i] = $crate::Seq::ConsRef( *item, $rt);
                }
                // element in the middle, located in array[idx] and referencing previous one idx-1
                else {
                    unsafe { // using raw-ptr as assignment to borrowed array is not possible
                      let pre: *const Seq<$typ> = &_a[i-1];
                      _a[i] = $crate::Seq::ConsRef( *item, &*pre);
                   }
                }
            }
            }
            top
        };
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

    // for seqdef_try
    use std::ptr;
    use std::mem;

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


        seqdef_try!(x, i32, 4; empty() => [ 0i32, 1, 2, 3].iter());
        assert_eq!(x.ok(), Some(u));

        let empty_list: &[i32] = &[];
        // failing, the list must contain one element at least
        seqdef_try!(x, i32, 4; empty() => empty_list.iter());
        assert_eq!(x.is_err(), true);

        // exceeding max elements 4
        seqdef_try!(x, i32, 4; empty() => [0i32, 1, 2, 3, 4].iter());
        assert_eq!(x.is_err(), true);


        // large number of elements 2000
        const LARGE_LIST_LEN: usize = 2000;
        let large_list: &[i32] = &[42; LARGE_LIST_LEN];
        seqdef_try!(x, i32, LARGE_LIST_LEN; empty() => large_list.iter());
        assert_eq!(x.is_ok(), true);
        match &x {
            &Ok(ref sequence) => println!("large sum {}", (&*sequence).into_iter().fold(0i32, ops::Add::add)),
            &Err(reason) => println!("roll out failed due to {}", reason)
        }
        assert_eq!(x.unwrap().into_iter().fold(0i32, ops::Add::add), 2000*42);
    }
}
