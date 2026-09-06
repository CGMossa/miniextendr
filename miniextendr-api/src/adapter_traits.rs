//! Built-in adapter traits for common Rust standard library traits.
//!
//! These traits provide blanket implementations that allow any Rust type
//! implementing standard traits to be exposed to R without boilerplate.
//!
//! # Example
//!
//! ```rust,ignore
//! use miniextendr_api::prelude::*;
//! use miniextendr_api::adapter_traits::RDebug;
//!
//! #[derive(Debug, ExternalPtr)]
//! struct MyData {
//!     values: Vec<i32>,
//! }
//!
//! // RDebug is automatically available for any Debug type
//! #[miniextendr]
//! impl RDebug for MyData {}
//! ```
//!
//! In R:
//! ```r
//! data <- MyData$new(...)
//! data$debug_str()        # "MyData { values: [1, 2, 3] }"
//! data$debug_str_pretty() # Pretty-printed with newlines
//! ```

use crate::miniextendr;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// Adapter trait for [`std::fmt::Debug`].
///
/// Provides string representations for debugging and inspection in R.
/// Automatically implemented for any type that implements `Debug`.
///
/// # Methods
///
/// - `debug_str()` - Returns compact debug string (`:?` format)
/// - `debug_str_pretty()` - Returns pretty-printed debug string (`:#?` format)
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Debug, ExternalPtr)]
/// struct Config { name: String, value: i32 }
///
/// #[miniextendr]
/// impl RDebug for Config {}
/// ```
#[miniextendr]
pub trait RDebug {
    /// Get a compact debug string representation.
    fn debug_str(&self) -> String;

    /// Get a pretty-printed debug string with indentation.
    fn debug_str_pretty(&self) -> String;
}

impl<T: Debug> RDebug for T {
    fn debug_str(&self) -> String {
        format!("{:?}", self)
    }

    fn debug_str_pretty(&self) -> String {
        format!("{:#?}", self)
    }
}

/// Adapter trait for [`std::fmt::Display`].
///
/// Provides user-friendly string conversion for R.
/// Automatically implemented for any type that implements `Display`.
///
/// # Methods
///
/// - `as_r_string()` - Returns the Display representation
///
/// # Example
///
/// ```rust,ignore
/// struct Version(u32, u32, u32);
///
/// impl Display for Version {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "{}.{}.{}", self.0, self.1, self.2)
///     }
/// }
///
/// #[miniextendr]
/// impl RDisplay for Version {}
/// ```
#[miniextendr]
pub trait RDisplay {
    /// Convert to a user-friendly string.
    fn as_r_string(&self) -> String;
}

impl<T: Display> RDisplay for T {
    fn as_r_string(&self) -> String {
        self.to_string()
    }
}

/// Adapter trait for [`std::hash::Hash`].
///
/// Provides hashing for deduplication and environment keys in R.
/// Automatically implemented for any type that implements `Hash`.
///
/// # Methods
///
/// - `hash()` - Returns the 64-bit hash as a 16-character hex string
///
/// # Note
///
/// Hash values are deterministic within a single R session but may vary
/// between sessions due to Rust's hasher implementation.
///
/// The hash is returned as a hex `String` (e.g. `"a1b2c3d4e5f60718"`), not a
/// number. R has no faithful 64-bit integer type, so returning the `u64` as a
/// numeric would round away its low bits above `2^53` — distinct Rust values
/// would collide in R — and surface half the range as negative. A hex string
/// preserves all 64 bits, is directly usable as an R environment key, and
/// works with `duplicated()` / `match()` for deduplication.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Hash, ExternalPtr)]
/// struct Record { id: String, value: i64 }
///
/// #[miniextendr]
/// impl RHash for Record {}
/// ```
#[miniextendr]
pub trait RHash {
    /// Compute a hash of this value.
    ///
    /// Returns the [`DefaultHasher`] `u64` output as a 16-character lowercase
    /// hex string. See the trait-level docs for why a string rather than a
    /// number.
    fn hash(&self) -> String;
}

impl<T: Hash> RHash for T {
    fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        // R has no faithful 64-bit integer; hex-encode the full u64 so every
        // bit survives the trip to R (a numeric would round above 2^53).
        format!("{:016x}", hasher.finish())
    }
}

/// Adapter trait for [`std::cmp::Ord`].
///
/// Provides total ordering comparison for R sorting operations.
/// Automatically implemented for any type that implements `Ord`.
///
/// # Methods
///
/// - `cmp(&self, other: &Self)` - Returns -1, 0, or 1
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Ord, PartialOrd, Eq, PartialEq, ExternalPtr)]
/// struct Priority(u32);
///
/// #[miniextendr]
/// impl ROrd for Priority {}
/// ```
#[miniextendr]
pub trait ROrd {
    /// Compare with another value.
    ///
    /// Returns:
    /// - `-1` if `self < other`
    /// - `0` if `self == other`
    /// - `1` if `self > other`
    fn cmp(&self, other: &Self) -> i32;
}

impl<T: Ord> ROrd for T {
    fn cmp(&self, other: &Self) -> i32 {
        match self.cmp(other) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}

/// Adapter trait for [`std::cmp::PartialOrd`].
///
/// Provides partial ordering comparison for R, handling incomparable values.
/// Automatically implemented for any type that implements `PartialOrd`.
///
/// # Methods
///
/// - `partial_cmp(&self, other: &Self)` - Returns Some(-1/0/1) or None
///
/// # Example
///
/// ```rust,ignore
/// // f64 has partial ordering (NaN is not comparable)
/// #[derive(PartialOrd, PartialEq, ExternalPtr)]
/// struct MyFloat(f64);
///
/// #[miniextendr]
/// impl RPartialOrd for MyFloat {}
/// ```
#[miniextendr]
pub trait RPartialOrd {
    /// Compare with another value, returning None if incomparable.
    ///
    /// Returns:
    /// - `Some(-1)` if `self < other`
    /// - `Some(0)` if `self == other`
    /// - `Some(1)` if `self > other`
    /// - `None` if values are incomparable (maps to NA in R)
    fn partial_cmp(&self, other: &Self) -> Option<i32>;
}

impl<T: PartialOrd> RPartialOrd for T {
    fn partial_cmp(&self, other: &Self) -> Option<i32> {
        self.partial_cmp(other).map(|ord| match ord {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }
}

/// Adapter trait for [`std::error::Error`].
///
/// Provides error message extraction and error chain walking for R.
/// Automatically implemented for any type that implements `Error`.
///
/// # Methods
///
/// - `error_message()` - Returns the error's display message
/// - `error_chain()` - Returns all messages in the error chain
///
/// # Example
///
/// ```rust,ignore
/// use std::error::Error;
/// use std::fmt;
///
/// #[derive(Debug)]
/// struct MyError { msg: String, source: Option<Box<dyn Error + Send + Sync>> }
///
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "{}", self.msg)
///     }
/// }
///
/// impl Error for MyError {
///     fn source(&self) -> Option<&(dyn Error + 'static)> {
///         self.source.as_ref().map(|e| e.as_ref() as _)
///     }
/// }
///
/// // Wrap in ExternalPtr for R access
/// #[derive(ExternalPtr)]
/// struct MyErrorWrapper(MyError);
///
/// #[miniextendr]
/// impl RError for MyErrorWrapper {}
/// ```
#[miniextendr]
pub trait RError {
    /// Get the error message (Display representation).
    fn error_message(&self) -> String;

    /// Get all error messages in the chain, from outermost to innermost.
    fn error_chain(&self) -> Vec<String>;

    /// Get the number of errors in the chain.
    fn error_chain_length(&self) -> i32;
}

impl<T: std::error::Error> RError for T {
    fn error_message(&self) -> String {
        self.to_string()
    }

    fn error_chain(&self) -> Vec<String> {
        let mut chain = vec![self.to_string()];
        let mut current: &dyn std::error::Error = self;
        while let Some(source) = current.source() {
            chain.push(source.to_string());
            current = source;
        }
        chain
    }

    fn error_chain_length(&self) -> i32 {
        let mut count = 1i32;
        let mut current: &dyn std::error::Error = self;
        while let Some(source) = current.source() {
            count += 1;
            current = source;
        }
        count
    }
}

/// Adapter trait for [`std::str::FromStr`].
///
/// Provides string parsing for R, allowing R strings to be parsed into Rust types.
/// Automatically implemented for any type that implements `FromStr`.
///
/// # Methods
///
/// - `from_str(s: &str)` - Parse a string into this type, returning None on failure
///
/// # Example
///
/// ```rust,ignore
/// use std::net::IpAddr;
///
/// // IpAddr implements FromStr
/// #[derive(ExternalPtr)]
/// struct IpAddress(IpAddr);
///
/// #[miniextendr]
/// impl RFromStr for IpAddress {}
/// ```
///
/// In R:
/// ```r
/// ip <- IpAddress$from_str("192.168.1.1")
/// ```
#[miniextendr]
pub trait RFromStr: Sized {
    /// Parse a string into this type.
    ///
    /// Returns `Some(value)` on success, `None` on parse failure.
    /// The None case maps to NULL in R.
    fn from_str(s: &str) -> Option<Self>;
}

impl<T: FromStr> RFromStr for T {
    fn from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

/// Adapter trait for [`std::clone::Clone`].
///
/// Provides explicit deep copying for R. This is useful when R users need
/// to create independent copies of Rust objects (which normally use reference
/// semantics via `ExternalPtr`).
///
/// # Methods
///
/// - `clone()` - Create a deep copy of this value
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Clone, ExternalPtr)]
/// struct Buffer { data: Vec<u8> }
///
/// #[miniextendr]
/// impl RClone for Buffer {}
/// ```
///
/// In R:
/// ```r
/// buf1 <- Buffer$new(...)
/// buf2 <- buf1$clone()  # Independent copy
/// ```
#[miniextendr]
pub trait RClone {
    /// Create a deep copy of this value.
    fn clone(&self) -> Self;
}

impl<T: Clone> RClone for T {
    fn clone(&self) -> Self {
        self.clone()
    }
}

/// Adapter trait for [`std::default::Default`].
///
/// Provides default value construction for R. This allows R users to create
/// instances with default values without needing to specify all parameters.
///
/// # Methods
///
/// - `default()` - Create a new instance with default values
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Default, ExternalPtr)]
/// struct Config {
///     timeout: u32,     // defaults to 0
///     retries: u32,     // defaults to 0
///     verbose: bool,    // defaults to false
/// }
///
/// #[miniextendr]
/// impl RDefault for Config {}
/// ```
///
/// In R:
/// ```r
/// config <- Config$default()  # All fields have default values
/// ```
#[miniextendr]
pub trait RDefault {
    /// Create a new instance with default values.
    fn default() -> Self;
}

impl<T: Default> RDefault for T {
    fn default() -> Self {
        Self::default()
    }
}

/// Adapter trait for [`std::marker::Copy`].
///
/// Indicates that a type can be cheaply copied (bitwise copy, no heap allocation).
/// This is useful for R users to know that copying is O(1) and doesn't involve
/// deep cloning of heap data.
///
/// # Methods
///
/// - `copy()` - Create a bitwise copy of this value
/// - `is_copy()` - Returns true (useful for runtime type checking in R)
///
/// # Difference from RClone
///
/// Both `RCopy` and `RClone` create copies, but:
/// - `RCopy`: Only for types where copying is cheap (stack-only, no heap)
/// - `RClone`: For any clonable type (may involve heap allocation)
///
/// If a type implements both, prefer `copy()` when you know copies are frequent.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Copy, Clone, ExternalPtr)]
/// struct Point { x: f64, y: f64 }
///
/// #[miniextendr]
/// impl RCopy for Point {}
/// ```
///
/// In R:
/// ```r
/// p1 <- Point$new(1.0, 2.0)
/// p2 <- p1$copy()  # Cheap bitwise copy
/// p1$is_copy()       # TRUE
/// ```
#[miniextendr]
pub trait RCopy {
    /// Create a bitwise copy of this value.
    ///
    /// For Copy types, this is always cheap (O(1), no heap allocation).
    fn copy(&self) -> Self;

    /// Check if this type implements Copy.
    ///
    /// Always returns true for types implementing this trait.
    /// Useful for runtime type checking in R.
    fn is_copy(&self) -> bool;
}

impl<T: Copy> RCopy for T {
    fn copy(&self) -> Self {
        *self
    }

    fn is_copy(&self) -> bool {
        true
    }
}

/// Adapter trait for [`std::iter::Iterator`].
///
/// Provides iterator operations for R, allowing Rust iterators to be consumed
/// element-by-element from R code. Since iterators are stateful, the wrapper
/// type should use interior mutability (e.g., `RefCell`).
///
/// # Methods
///
/// - `next()` - Get the next element, or None if exhausted
/// - `size_hint()` - Get estimated remaining elements as `c(lower, upper)`
/// - `count()` - Consume and count remaining elements
/// - `collect_n(n)` - Collect up to n elements into a vector
/// - `skip(n)` - Skip n elements
/// - `nth(n)` - Get the nth element (0-indexed)
///
/// # Example
///
/// ```rust,ignore
/// use std::cell::RefCell;
///
/// #[derive(ExternalPtr)]
/// struct MyIter(RefCell<std::vec::IntoIter<i32>>);
///
/// impl MyIter {
///     fn new(data: Vec<i32>) -> Self {
///         Self(RefCell::new(data.into_iter()))
///     }
/// }
///
/// impl RIterator for MyIter {
///     type Item = i32;
///
///     fn next(&self) -> Option<Self::Item> {
///         self.0.borrow_mut().next()
///     }
///
///     fn size_hint(&self) -> (i64, Option<i64>) {
///         let (lo, hi) = self.0.borrow().size_hint();
///         (lo as i64, hi.map(|h| h as i64))
///     }
/// }
///
/// #[miniextendr]
/// impl RIterator for MyIter {}
/// ```
///
/// In R (note: `next` is a reserved word, so expose as `next_item` or similar):
/// ```r
/// it <- MyIter$new(c(1L, 2L, 3L))
/// it$next_item()   # 1L
/// it$next_item()   # 2L
/// it$size_hint()   # c(1, 1) - one element remaining
/// it$next_item()   # 3L
/// it$next_item()   # NULL (exhausted)
/// ```
///
/// # Design Note
///
/// Unlike other adapter traits, `RIterator` does NOT have a blanket impl
/// because iterators require `&mut self` for `next()`, but R's ExternalPtr
/// pattern typically provides `&self`. Users must implement this trait
/// manually using interior mutability (RefCell, Mutex, etc.).
#[miniextendr]
pub trait RIterator {
    /// The type of elements yielded by this iterator.
    type Item;

    /// Get the next element from the iterator.
    ///
    /// Returns `Some(item)` if there are more elements, `None` if exhausted.
    /// None maps to NULL in R.
    #[miniextendr(r_name = "next_item")]
    fn next(&self) -> Option<Self::Item>;

    /// Get the estimated number of remaining elements.
    ///
    /// Returns `(lower_bound, upper_bound)` where upper_bound is None if unknown.
    /// In R, this becomes `c(lower, upper)` where upper is NA if unknown.
    ///
    /// Skipped from trait ABI because tuples don't have R conversions.
    /// Expose via manual forwarding or custom wrapper methods.
    #[miniextendr(skip)]
    fn size_hint(&self) -> (i64, Option<i64>);

    /// Consume the iterator and count remaining elements.
    ///
    /// **Warning:** This exhausts the iterator.
    fn count(&self) -> i64 {
        let mut count = 0i64;
        while self.next().is_some() {
            count += 1;
        }
        count
    }

    /// Collect up to `n` elements into a vector.
    ///
    /// Returns fewer than `n` elements if the iterator is exhausted first.
    fn collect_n(&self, n: i32) -> Vec<Self::Item> {
        let mut result = Vec::with_capacity(n.max(0) as usize);
        for _ in 0..n {
            match self.next() {
                Some(item) => result.push(item),
                None => break,
            }
        }
        result
    }

    /// Skip `n` elements from the iterator.
    ///
    /// Returns the number of elements actually skipped (may be less than `n`
    /// if the iterator is exhausted).
    fn skip(&self, n: i32) -> i32 {
        let mut skipped = 0i32;
        for _ in 0..n {
            if self.next().is_none() {
                break;
            }
            skipped += 1;
        }
        skipped
    }

    /// Get the `n`th element (0-indexed), consuming elements up to and including it.
    ///
    /// Returns None if the iterator has fewer than `n + 1` elements.
    fn nth(&self, n: i32) -> Option<Self::Item> {
        if n < 0 {
            return None;
        }
        for _ in 0..n {
            self.next()?;
        }
        self.next()
    }
}

// Note: No blanket impl because Iterator::next() requires &mut self,
// but ExternalPtr methods receive &self. Users must use interior mutability.

/// Adapter trait for [`std::iter::Extend`].
///
/// Provides collection extension operations for R, allowing Rust collections
/// to be extended with R vectors. Since extension requires mutation, the
/// wrapper type should use interior mutability (e.g., `RefCell`).
///
/// # Methods
///
/// - `extend_from_vec(items)` - Extend the collection with items from a vector
/// - `extend_from_slice(items)` - Extend from a slice (for Clone items)
///
/// # Example
///
/// ```rust,ignore
/// use std::cell::RefCell;
///
/// #[derive(ExternalPtr)]
/// struct MyVec(RefCell<Vec<i32>>);
///
/// impl MyVec {
///     fn new() -> Self {
///         Self(RefCell::new(Vec::new()))
///     }
/// }
///
/// impl RExtend<i32> for MyVec {
///     fn extend_from_vec(&self, items: Vec<i32>) {
///         self.0.borrow_mut().extend(items);
///     }
/// }
///
/// #[miniextendr]
/// impl RExtend<i32> for MyVec {}
/// ```
///
/// In R:
/// ```r
/// v <- MyVec$new()
/// v$extend_from_vec(c(1L, 2L, 3L))  # Add items
/// v$extend_from_vec(c(4L, 5L))      # Add more items
/// ```
///
/// # Design Note
///
/// Like `RIterator`, `RExtend` does NOT have a blanket impl because `Extend::extend()`
/// requires `&mut self`, but R's ExternalPtr pattern provides `&self`. Users must
/// implement this trait manually using interior mutability (RefCell, Mutex, etc.).
#[miniextendr]
pub trait RExtend<T> {
    /// Extend the collection with items from a vector.
    ///
    /// The items are moved into the collection.
    fn extend_from_vec(&self, items: Vec<T>);

    /// Extend the collection with cloned items from a slice.
    ///
    /// Default implementation clones items into a Vec and calls `extend_from_vec`.
    ///
    /// Skipped from trait ABI because `&[T]` doesn't have TryFromSexp.
    #[miniextendr(skip)]
    fn extend_from_slice(&self, items: &[T])
    where
        T: Clone,
    {
        self.extend_from_vec(items.to_vec());
    }

    /// Get the current length of the collection.
    ///
    /// Optional - returns -1 if not implemented.
    fn len(&self) -> i64 {
        -1 // Indicates "unknown" - implementers can override
    }

    /// Check if the collection is empty.
    ///
    /// Returns false when length is unknown.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Note: No blanket impl because Extend::extend() requires &mut self,
// but ExternalPtr methods receive &self. Users must use interior mutability.

/// Adapter trait for [`std::iter::FromIterator`].
///
/// Provides collection construction from iterators/vectors for R.
/// Unlike `RExtend`, this creates a new collection from items.
///
/// # Methods
///
/// - `from_vec(items)` - Create a new collection from a vector
///
/// # Example
///
/// ```rust,ignore
/// #[derive(ExternalPtr)]
/// struct MySet(std::collections::HashSet<i32>);
///
/// impl RFromIter<i32> for MySet {
///     fn from_vec(items: Vec<i32>) -> Self {
///         Self(items.into_iter().collect())
///     }
/// }
///
/// #[miniextendr]
/// impl RFromIter<i32> for MySet {}
/// ```
///
/// In R:
/// ```r
/// set <- MySet$from_vec(c(1L, 2L, 2L, 3L))  # Creates {1, 2, 3}
/// ```
#[miniextendr]
pub trait RFromIter<T>: Sized {
    /// Create a new collection from a vector of items.
    fn from_vec(items: Vec<T>) -> Self;
}

impl<C, T> RFromIter<T> for C
where
    C: FromIterator<T>,
{
    fn from_vec(items: Vec<T>) -> Self {
        items.into_iter().collect()
    }
}

/// Adapter trait for collections that can be converted to vectors.
///
/// This is the complement to [`RFromIter`]: while `RFromIter` creates collections
/// from vectors, `RToVec` extracts vectors from collections.
///
/// # Methods
///
/// - `to_vec()` - Collect all elements into a vector (cloning elements)
/// - `len()` - Get the number of elements
/// - `is_empty()` - Check if the collection is empty
///
/// # Design Note
///
/// Unlike Rust's `IntoIterator::into_iter()` which consumes the collection,
/// this trait borrows the collection and clones elements. This is necessary
/// because R's `ExternalPtr` pattern provides `&self`, not owned `self`.
///
/// For consuming iteration, use [`RIterator`] with interior mutability.
///
/// # Example
///
/// ```rust,ignore
/// use std::collections::HashSet;
///
/// #[derive(ExternalPtr)]
/// struct MySet(HashSet<i32>);
///
/// // RToVec is automatically available via blanket impl
/// #[miniextendr]
/// impl RToVec<i32> for MySet {}
/// ```
///
/// In R:
/// ```r
/// set <- MySet$new(...)
/// vec <- set$to_vec()    # Get all elements as vector
/// set$len()              # Number of elements
/// set$is_empty()         # Check if empty
/// ```
#[miniextendr]
pub trait RToVec<T> {
    /// Collect all elements into a vector.
    ///
    /// Elements are cloned from the collection.
    fn to_vec(&self) -> Vec<T>;

    /// Get the number of elements in the collection.
    fn len(&self) -> i64;

    /// Check if the collection is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Blanket impl for any collection where:
// - &C can be iterated over (yielding &T references)
// - T: Clone (so we can clone elements into the Vec)
// - The iterator knows its exact size
//
// Note: Using HRTB (higher-ranked trait bounds) to express that &C
// can be iterated for any lifetime.
impl<C, T> RToVec<T> for C
where
    T: Clone,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    for<'a> <&'a C as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn to_vec(&self) -> Vec<T> {
        self.into_iter().cloned().collect()
    }

    fn len(&self) -> i64 {
        self.into_iter().len() as i64
    }
}

/// Adapter trait for creating iterator wrappers from collections.
///
/// This trait provides a way to create an [`RIterator`] wrapper from a collection.
/// Since `ExternalPtr` methods receive `&self`, this trait clones the underlying
/// data to create an independent iterator.
///
/// # Type Parameters
///
/// - `T`: The element type yielded by the iterator
/// - `I`: The iterator type returned (must implement [`RIterator`])
///
/// # Design Note
///
/// The returned iterator is independent from the source collection. Modifications
/// to the original collection after calling `make_iter()` won't affect the
/// iterator's output.
///
/// # Example
///
/// ```rust,ignore
/// use std::cell::RefCell;
///
/// #[derive(ExternalPtr)]
/// struct MyVec(Vec<i32>);
///
/// #[derive(ExternalPtr)]
/// struct MyVecIter(RefCell<std::vec::IntoIter<i32>>);
///
/// impl RIterator for MyVecIter {
///     type Item = i32;
///     fn next(&self) -> Option<i32> {
///         self.0.borrow_mut().next()
///     }
///     fn size_hint(&self) -> (i64, Option<i64>) {
///         let (lo, hi) = self.0.borrow().size_hint();
///         (lo as i64, hi.map(|h| h as i64))
///     }
/// }
///
/// impl RMakeIter<i32, MyVecIter> for MyVec {
///     fn make_iter(&self) -> MyVecIter {
///         MyVecIter(RefCell::new(self.0.clone().into_iter()))
///     }
/// }
///
/// #[miniextendr]
/// impl RMakeIter<i32, MyVecIter> for MyVec {}
/// ```
///
/// In R (note: expose `next` as `next_item` since `next` is reserved):
/// ```r
/// v <- MyVec$new(c(1L, 2L, 3L))
/// it <- v$make_iter()   # Create iterator
/// it$next_item()        # 1L
/// it$next_item()        # 2L
/// v$to_vec()            # c(1L, 2L, 3L) - original unchanged
/// ```
#[miniextendr]
pub trait RMakeIter<T, I>
where
    I: RIterator<Item = T>,
{
    /// Create a new iterator wrapper.
    ///
    /// The iterator is independent from this collection (typically by cloning
    /// the underlying data).
    fn make_iter(&self) -> I;
}

// Note: No blanket impl because:
// 1. The iterator type I must be a concrete type that implements RIterator
// 2. RIterator requires interior mutability (RefCell/Mutex)
// 3. Users must define their own iterator wrapper type

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdebug() {
        let v = vec![1, 2, 3];
        assert_eq!(v.debug_str(), "[1, 2, 3]");
        assert!(v.debug_str_pretty().contains('\n') || v.debug_str_pretty() == "[1, 2, 3]");
    }

    #[test]
    fn test_rdisplay() {
        let s = "hello";
        assert_eq!(s.as_r_string(), "hello");

        let n = 42i32;
        assert_eq!(n.as_r_string(), "42");
    }

    #[test]
    fn test_rhash() {
        let a = "test";
        let b = "test";
        let c = "other";

        assert_eq!(RHash::hash(&a), RHash::hash(&b));
        assert_ne!(RHash::hash(&a), RHash::hash(&c));
    }

    #[test]
    fn test_rord() {
        assert_eq!(ROrd::cmp(&1i32, &2), -1);
        assert_eq!(ROrd::cmp(&2i32, &2), 0);
        assert_eq!(ROrd::cmp(&3i32, &2), 1);
    }

    #[test]
    fn test_rpartialord() {
        assert_eq!(RPartialOrd::partial_cmp(&1.0f64, &2.0), Some(-1));
        assert_eq!(RPartialOrd::partial_cmp(&2.0f64, &2.0), Some(0));
        assert_eq!(RPartialOrd::partial_cmp(&3.0f64, &2.0), Some(1));
        assert_eq!(RPartialOrd::partial_cmp(&f64::NAN, &1.0), None);
    }

    #[test]
    fn test_rerror_simple() {
        use std::io;
        let err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        assert_eq!(err.error_message(), "file not found");
        assert_eq!(err.error_chain().len(), 1);
        assert_eq!(err.error_chain_length(), 1);
    }

    #[test]
    fn test_rerror_chain() {
        use std::fmt;

        #[derive(Debug)]
        struct OuterError {
            msg: &'static str,
            source: InnerError,
        }

        #[derive(Debug)]
        struct InnerError {
            msg: &'static str,
        }

        impl fmt::Display for OuterError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.msg)
            }
        }

        impl fmt::Display for InnerError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.msg)
            }
        }

        impl std::error::Error for InnerError {}

        impl std::error::Error for OuterError {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.source)
            }
        }

        let err = OuterError {
            msg: "outer error",
            source: InnerError { msg: "inner error" },
        };

        assert_eq!(err.error_message(), "outer error");
        let chain = err.error_chain();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], "outer error");
        assert_eq!(chain[1], "inner error");
        assert_eq!(err.error_chain_length(), 2);
    }

    #[test]
    fn test_rfromstr_success() {
        let result: Option<i32> = RFromStr::from_str("42");
        assert_eq!(result, Some(42));

        let result: Option<f64> = RFromStr::from_str("3.141592653589793");
        assert_eq!(result, Some(core::f64::consts::PI));

        let result: Option<bool> = RFromStr::from_str("true");
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_rfromstr_failure() {
        let result: Option<i32> = RFromStr::from_str("not a number");
        assert_eq!(result, None);

        let result: Option<f64> = RFromStr::from_str("abc");
        assert_eq!(result, None);
    }

    #[test]
    fn test_rclone() {
        let v = vec![1, 2, 3];
        let cloned = RClone::clone(&v);
        assert_eq!(v, cloned);

        // Verify it's a deep copy
        let s = String::from("hello");
        let cloned_s = RClone::clone(&s);
        assert_eq!(s, cloned_s);
    }

    #[test]
    fn test_rdefault() {
        let default_i32: i32 = RDefault::default();
        assert_eq!(default_i32, 0);

        let default_vec: Vec<i32> = RDefault::default();
        assert!(default_vec.is_empty());

        let default_string: String = RDefault::default();
        assert_eq!(default_string, "");

        let default_bool: bool = RDefault::default();
        assert!(!default_bool);
    }

    #[test]
    fn test_rcopy() {
        // Primitives are Copy
        let x = 42i32;
        let y = RCopy::copy(&x);
        assert_eq!(x, y);
        assert!(x.is_copy());

        // Tuples of Copy types are Copy
        let point = (1.0f64, 2.0f64);
        let point2 = RCopy::copy(&point);
        assert_eq!(point, point2);
        assert!(point.is_copy());

        // Arrays of Copy types are Copy
        let arr = [1, 2, 3];
        let arr2 = RCopy::copy(&arr);
        assert_eq!(arr, arr2);
    }

    // Tests for RIterator
    use std::cell::RefCell;

    /// Test iterator wrapper using RefCell for interior mutability.
    struct TestIter(RefCell<std::vec::IntoIter<i32>>);

    impl TestIter {
        fn new(data: Vec<i32>) -> Self {
            Self(RefCell::new(data.into_iter()))
        }
    }

    impl RIterator for TestIter {
        type Item = i32;

        fn next(&self) -> Option<Self::Item> {
            self.0.borrow_mut().next()
        }

        fn size_hint(&self) -> (i64, Option<i64>) {
            let (lo, hi) = self.0.borrow().size_hint();
            (lo as i64, hi.map(|h| h as i64))
        }
    }

    #[test]
    fn test_riterator_next() {
        let it = TestIter::new(vec![1, 2, 3]);
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next(), Some(2));
        assert_eq!(it.next(), Some(3));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None); // Stays exhausted
    }

    #[test]
    fn test_riterator_size_hint() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(it.size_hint(), (5, Some(5)));
        it.next();
        assert_eq!(it.size_hint(), (4, Some(4)));
        it.next();
        it.next();
        assert_eq!(it.size_hint(), (2, Some(2)));
    }

    #[test]
    fn test_riterator_count() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(it.count(), 5);
        // Iterator is now exhausted
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_riterator_collect_n() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        let first_three = it.collect_n(3);
        assert_eq!(first_three, vec![1, 2, 3]);
        let remaining = it.collect_n(10); // Ask for more than available
        assert_eq!(remaining, vec![4, 5]);
    }

    #[test]
    fn test_riterator_skip() {
        let it = TestIter::new(vec![1, 2, 3, 4, 5]);
        let skipped = it.skip(2);
        assert_eq!(skipped, 2);
        assert_eq!(it.next(), Some(3));

        // Skip more than remaining
        let skipped = it.skip(10);
        assert_eq!(skipped, 2); // Only 2 elements were left
    }

    #[test]
    fn test_riterator_nth() {
        let it = TestIter::new(vec![10, 20, 30, 40, 50]);
        // Get element at index 2 (third element)
        assert_eq!(it.nth(2), Some(30));
        // Iterator has consumed 0, 1, 2 - next is index 3
        assert_eq!(it.next(), Some(40));

        // Negative index returns None
        let it2 = TestIter::new(vec![1, 2, 3]);
        assert_eq!(it2.nth(-1), None);
    }

    #[test]
    fn test_riterator_empty() {
        let it = TestIter::new(vec![]);
        assert_eq!(it.next(), None);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.count(), 0);
        assert_eq!(it.collect_n(5), Vec::<i32>::new());
        assert_eq!(it.skip(5), 0);
        assert_eq!(it.nth(0), None);
    }

    // Tests for RExtend
    struct TestExtendVec(RefCell<Vec<i32>>);

    impl TestExtendVec {
        fn new() -> Self {
            Self(RefCell::new(Vec::new()))
        }

        fn get(&self) -> Vec<i32> {
            Clone::clone(&*self.0.borrow())
        }
    }

    impl RExtend<i32> for TestExtendVec {
        fn extend_from_vec(&self, items: Vec<i32>) {
            self.0.borrow_mut().extend(items);
        }

        fn len(&self) -> i64 {
            self.0.borrow().len() as i64
        }
    }

    #[test]
    fn test_rextend_basic() {
        let v = TestExtendVec::new();
        assert_eq!(v.get(), Vec::<i32>::new());
        assert_eq!(v.len(), 0);

        v.extend_from_vec(vec![1, 2, 3]);
        assert_eq!(v.get(), vec![1, 2, 3]);
        assert_eq!(v.len(), 3);

        v.extend_from_vec(vec![4, 5]);
        assert_eq!(v.get(), vec![1, 2, 3, 4, 5]);
        assert_eq!(v.len(), 5);
    }

    #[test]
    fn test_rextend_empty() {
        let v = TestExtendVec::new();
        v.extend_from_vec(vec![]);
        assert_eq!(v.get(), Vec::<i32>::new());
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_rextend_from_slice() {
        let v = TestExtendVec::new();
        let data = [1, 2, 3];
        v.extend_from_slice(&data);
        assert_eq!(v.get(), vec![1, 2, 3]);
    }

    // Tests for RFromIter
    #[test]
    fn test_rfromiter_vec() {
        let v: Vec<i32> = RFromIter::from_vec(vec![1, 2, 3]);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_rfromiter_hashset() {
        use std::collections::HashSet;
        let set: HashSet<i32> = RFromIter::from_vec(vec![1, 2, 2, 3, 3, 3]);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
    }

    #[test]
    fn test_rfromiter_string() {
        let s: String = RFromIter::from_vec(vec!['h', 'e', 'l', 'l', 'o']);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_rfromiter_empty() {
        let v: Vec<i32> = RFromIter::from_vec(vec![]);
        assert!(v.is_empty());
    }

    // Tests for RToVec
    #[test]
    fn test_rtovec_vec() {
        let v = vec![1, 2, 3];
        let collected: Vec<i32> = RToVec::to_vec(&v);
        assert_eq!(collected, vec![1, 2, 3]);
        assert_eq!(RToVec::<i32>::len(&v), 3);
        assert!(!RToVec::<i32>::is_empty(&v));
    }

    #[test]
    fn test_rtovec_empty() {
        let v: Vec<i32> = vec![];
        let collected: Vec<i32> = RToVec::to_vec(&v);
        assert!(collected.is_empty());
        assert_eq!(RToVec::<i32>::len(&v), 0);
        assert!(RToVec::<i32>::is_empty(&v));
    }

    #[test]
    fn test_rtovec_hashset() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let mut collected: Vec<i32> = RToVec::to_vec(&set);
        collected.sort();
        assert_eq!(collected, vec![1, 2, 3]);
        assert_eq!(RToVec::<i32>::len(&set), 3);
    }

    #[test]
    fn test_rtovec_slice() {
        let arr = [10, 20, 30];
        let collected: Vec<i32> = RToVec::to_vec(&arr);
        assert_eq!(collected, vec![10, 20, 30]);
        assert_eq!(RToVec::<i32>::len(&arr), 3);
    }

    // Tests for RMakeIter
    struct TestCollection(Vec<i32>);

    struct TestCollectionIter(RefCell<std::vec::IntoIter<i32>>);

    impl RIterator for TestCollectionIter {
        type Item = i32;

        fn next(&self) -> Option<i32> {
            self.0.borrow_mut().next()
        }

        fn size_hint(&self) -> (i64, Option<i64>) {
            let (lo, hi) = self.0.borrow().size_hint();
            (lo as i64, hi.map(|h| h as i64))
        }
    }

    impl RMakeIter<i32, TestCollectionIter> for TestCollection {
        fn make_iter(&self) -> TestCollectionIter {
            TestCollectionIter(RefCell::new(Clone::clone(&self.0).into_iter()))
        }
    }

    #[test]
    fn test_rmakeiter_basic() {
        let coll = TestCollection(vec![1, 2, 3]);
        let iter = coll.make_iter();

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_rmakeiter_independent() {
        let coll = TestCollection(vec![1, 2, 3]);

        // Create two independent iterators
        let iter1 = coll.make_iter();
        let iter2 = coll.make_iter();

        // Consuming one doesn't affect the other
        assert_eq!(iter1.next(), Some(1));
        assert_eq!(iter1.next(), Some(2));

        assert_eq!(iter2.next(), Some(1)); // iter2 starts fresh
        assert_eq!(iter2.size_hint(), (2, Some(2))); // 2 remaining in iter2
    }
}
