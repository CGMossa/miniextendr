# ExternalPtr

`ExternalPtr<T>` is a Box-like owned pointer that wraps R's `EXTPTRSXP`. It lets you hand ownership of Rust-allocated data to R and let R's garbage collector decide when to drop it.

**Source**: `miniextendr-api/src/externalptr.rs`

## Why ExternalPtr Exists

R has no native way to hold arbitrary Rust data. `EXTPTRSXP` is R's mechanism for storing opaque C pointers, but it provides no type safety, no RAII cleanup, and no protection against use-after-free. `ExternalPtr<T>` wraps `EXTPTRSXP` with:

- **Type-safe access** via authoritative `Any::downcast` checks; the
  `TypedExternal` symbols are display and diagnostic metadata
- **Automatic cleanup** via R GC finalizer that calls `Drop`
- **Box-like API** (`Deref`, `DerefMut`, `Clone`, `into_inner`, `into_raw`, `pin`, etc.)
- **Context-checked construction** -- `new()` runs on R's main thread or routes
  from an active miniextendr worker context

## When to Use ExternalPtr

| Strategy | Lifetime | Use Case |
|----------|----------|----------|
| `ExternalPtr` | Until R GCs | Rust data owned by R (structs returned to R) |
| `ProtectScope` | Within `.Call` | Temporary R allocations |
| Preserve list | Across `.Call`s | Long-lived R objects (not Rust values) |

Use `ExternalPtr` when you want R to own a Rust value and drop it when R garbage-collects the pointer. This is the standard mechanism for exposing Rust structs to R code.

## Creating an ExternalPtr

### With `#[derive(ExternalPtr)]` (recommended)

The derive macro implements `TypedExternal` and `IntoExternalPtr`, so returning your struct from a `#[miniextendr]` function automatically wraps it:

```rust
#[derive(ExternalPtr)]
pub struct MyData {
    pub value: f64,
}

#[miniextendr]
pub fn create_data(v: f64) -> MyData {
    MyData { value: v }  // Automatically wrapped in ExternalPtr
}
```

### Manual construction

```rust
let ptr = ExternalPtr::new(MyData { value: 3.14 });
```

`new()` works on R's main thread and from miniextendr's dedicated worker during
an active `run_on_worker` call; the latter routes its R allocations through
`with_r_thread`. It panics on an arbitrary spawned or Rayon thread rather than
calling R off-main.

### Unchecked construction (ALTREP callbacks, main-thread-only code)

```rust
// SAFETY: must be on R's main thread
let ptr = unsafe { ExternalPtr::new_unchecked(MyData { value: 3.14 }) };
```

Skips main-thread routing and checks for performance-critical paths whose
caller has already established R's main-thread context.

### From raw pointers

```rust
let raw = Box::into_raw(Box::new(MyData { value: 1.0 }));
// SAFETY: raw was allocated by Box, is non-null, caller transfers ownership
let ptr = unsafe { ExternalPtr::from_raw(raw) };
```

## Passing Class Objects to Standalone Functions

A `#[miniextendr]` argument typed as `ExternalPtr<T>` accepts either the bare
pointer a low-level constructor returns, **or** the ergonomic class handle
that wraps it — `MyData$new(...)`, `S4Foo(...)`, etc. Conversion unwraps the
handle transparently, per class system:

| Class system | Where the pointer lives | Shape |
|---|---|---|
| Env, S3 | the object itself | already a classed `EXTPTRSXP` — nothing to unwrap |
| R6 | `private$.ptr` | `.__enclos_env__` -> `private` -> `.ptr` |
| S4 | the `ptr` slot | `methods::slot(x, "ptr")` |
| S7 | the `.ptr` property | stored as a plain attribute (`attr(x, ".ptr")`) |
| hand-built list | the `.ptr` element | `structure(list(.ptr = ptr, log = ...), class = "Foo")` |
| hand-built environment | a `.ptr` binding | any environment binding `.ptr` directly |

The same resolution applies to the receiver of every class-system instance
method (`&self`, `&mut self`, `self`, and the `ExternalPtr<Self>` receiver
forms): the generated prelude calls `resolve_receiver` on the object R
dispatched with, so an S3 class whose object is a list with R-side state next
to the handle dispatches into `#[miniextendr(s3)] impl` methods unchanged
(#1469; `S3_METHODS.md`, "Handles with R-visible state"). That shape is for
interop with R-defined objects: state the package owns belongs in sidecar
fields (`#[r_data]`, "RSidecar" below), which travel with the handle in every
class system. A bare pointer is recognised with one
`TYPEOF` compare; a receiver that yields no pointer raises
`expected a `Foo` object (an external pointer, or an R6/S4/S7 handle,
environment, or list carrying one in `.ptr`), got VECSXP`. Type safety is
unchanged: the pointer that comes out still has to `downcast` to `Foo`.

```rust
#[derive(ExternalPtr)]
pub struct MyData { pub value: f64 }

#[miniextendr(r6)]
impl MyData {
    pub fn new(value: f64) -> Self { Self { value } }
}

#[miniextendr]
pub fn total(items: Vec<ExternalPtr<MyData>>) -> f64 {
    items.iter().map(|d| d.value).sum()
}
```

```r
a <- MyData$new(1.0)
b <- MyData$new(2.0)
total(list(a, b))  # 3 -- no need to reach for a bare-pointer constructor
```

`Any::downcast` remains the type-safety authority: a handle for the *wrong*
Rust type still fails with the usual downcast type-mismatch error — this
only widens the accepted R-side shape, not type safety. Vctrs is the one
class system where this never applies: a vctrs object is a plain classed
vector with no Rust struct behind it (vctrs constructors return a payload
vector, not `Self`), so there's no `ExternalPtr` to unwrap in the first
place.

## Accessing the Value

`ExternalPtr<T>` implements `Deref<Target = T>` and `DerefMut`, so you can use it like a reference:

```rust
let ptr = ExternalPtr::new(MyData { value: 3.14 });
println!("{}", ptr.value);  // Deref to &MyData
```

Explicit access methods:

| Method | Returns | Notes |
|--------|---------|-------|
| `as_ref()` | `Option<&T>` | Always `Some` for valid ptrs |
| `as_mut()` | `Option<&mut T>` | Always `Some` for valid ptrs |
| `as_ptr()` | `*const T` | Raw pointer, no ownership transfer |
| `as_sexp()` | `SEXP` | The underlying R object |
| `reborrow()` | `ExternalPtr<T>` | Owned alias sharing the same SEXP; no allocation, no R object copy |

### `reborrow()`: identity-preserving returns

When a `#[miniextendr]` method receives one or more `ExternalPtr<T>` values
and returns one of them to R, `reborrow()` lets you build an owned
`ExternalPtr<T>` that points at the same `EXTPTRSXP` without allocating a
new R object:

```rust
#[miniextendr(env)]
impl Counter {
    // R will see `identical(a, pick_larger(a, b))` == TRUE when `a` wins,
    // because reborrow() returns the same SEXP rather than a fresh copy.
    pub fn pick_larger(
        self: &ExternalPtr<Self>,
        other: &ExternalPtr<Self>,
    ) -> ExternalPtr<Self> {
        if self.value >= other.value { self.reborrow() } else { other.reborrow() }
    }
}
```

`clone()` would allocate a fresh SEXP with a deep copy of the inner `T`;
`reborrow()` is the correct choice when the caller expects to get back the
same R object they passed in.

## Consuming and Dropping

| Method | Effect |
|--------|--------|
| `into_inner(this)` | Moves value out, deallocates, neutralizes finalizer |
| `into_raw(this)` | Returns `*mut T`, neutralizes finalizer, caller owns memory |
| `leak(this)` | Returns `&'a mut T`, memory is never freed |

R's GC finalizer handles cleanup when the `ExternalPtr` goes out of scope in Rust without being explicitly consumed. The Rust `Drop` impl is a no-op to avoid double-free.

### Panicking destructors are not recoverable

The GC finalizer is an `extern "C"` function. Unwinding through it is
undefined behavior. miniextendr wraps every ExternalPtr finalizer in
`drop_catching_panic`, which calls `std::panic::catch_unwind` around the
destructor; if a `Drop` impl panics the helper prints the panic message
to stderr and calls `std::process::abort()`. That's the only safe
response: R's GC cannot resume after a cross-ABI unwind.

This means: a panic in `impl Drop for YourType` crashes the R session.
Treat destructors as infallible. Put anything that can fail (file
writes, network, mutex takedown) behind an explicit `close()` /
`finalize()` method that the user calls, and keep `Drop` limited to
plain memory release.

## ExternalPtr as a Self Receiver

Inside a `#[miniextendr]` impl block, methods can take the wrapping
`ExternalPtr` as the receiver instead of a plain reference:

```rust
#[miniextendr(env)]
impl MyType {
    // Plain receiver - gets &MyType via Deref
    pub fn value(&self) -> i32 { self.value }

    // ExternalPtr receivers - access ExternalPtr methods directly;
    // Deref/DerefMut still expose the inner T transparently.
    pub fn is_null_ptr(self: &ExternalPtr<Self>) -> bool {
        self.is_null()
    }

    pub fn set_via_ptr(self: &mut ExternalPtr<Self>, v: i32) {
        self.value = v;  // DerefMut to &mut MyType
    }
}
```

This is the form to use when a method needs `ExternalPtr` identity or tag
metadata: `as_sexp()`, `tag()`, `protected()`, `ptr_eq()`, `reborrow()`,
etc. The macro rewrites `self` to an internal binding so the pattern
compiles on stable Rust (no `arbitrary_self_types` required), and the
generated C wrapper uses typed `ExternalPtr::<T>::wrap_sexp()` rather than
an erased downcast.

Allowed forms: `self: &ExternalPtr<Self>`, `self: &mut ExternalPtr<Self>`.
Consuming receivers (`self: ExternalPtr<Self>`) are not supported. R owns
the pointer.

## Display and Diagnostic Metadata with TypedExternal

Every `ExternalPtr<T>` requires `T: TypedExternal`. This trait provides two identifiers stored in the SEXP:

- **`TYPE_NAME_CSTR`** -- Short display name, stored in the `tag` slot (visible when printing in R)
- **`TYPE_ID_CSTR`** -- Namespaced identifier (`crate@version::module::Type`), stored in `prot[0]` for mismatch diagnostics

R interns both strings with `Rf_install`, but concrete type checking uses the
stored `Any` value's Rust `TypeId`. The symbols make R-facing names and errors
readable; they are not a substitute for the downcast.

### Implementing TypedExternal

**Via derive (recommended):**

```rust
#[derive(ExternalPtr)]
pub struct MyData { /* ... */ }
```

**Via macro:**

```rust
impl_typed_external!(MyData);
// also works for generic types:
impl_typed_external!(MyWrapper<i32>);
```

**Manually:**

```rust
impl TypedExternal for MyData {
    const TYPE_NAME: &'static str = "MyData";
    const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
    const TYPE_ID_CSTR: &'static [u8] =
        concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"),
                "::", module_path!(), "::MyData\0").as_bytes();
}
```

### Built-in TypedExternal Implementations

**Source**: `miniextendr-api/src/externalptr_std.rs`

The following standard library types have built-in `TypedExternal` impls, so they can be stored in `ExternalPtr<T>` without any manual implementation:

| Category | Types |
|----------|-------|
| **Primitives** | `bool`, `char`, `i8`–`i128`, `isize`, `u8`–`u128`, `usize`, `f32`, `f64` |
| **Strings** | `String`, `CString`, `OsString`, `PathBuf` |
| **Collections** | `Vec<T>`, `VecDeque<T>`, `LinkedList<T>`, `BinaryHeap<T>`, `HashMap<K,V>`, `BTreeMap<K,V>`, `HashSet<T>`, `BTreeSet<T>` |
| **Smart pointers** | `Box<T>`, `Box<[T]>`, `Rc<T>`, `Arc<T>`, `Cell<T>`, `RefCell<T>`, `UnsafeCell<T>`, `Mutex<T>`, `RwLock<T>`, `OnceLock<T>`, `Pin<T>`, `ManuallyDrop<T>`, `MaybeUninit<T>`, `PhantomData<T>` |
| **Option/Result** | `Option<T>`, `Result<T, E>` |
| **Ranges** | `Range<T>`, `RangeInclusive<T>`, `RangeFrom<T>`, `RangeTo<T>`, `RangeToInclusive<T>`, `RangeFull` |
| **I/O** | `File`, `BufReader<R>`, `BufWriter<W>`, `Cursor<T>` |
| **Time** | `Duration`, `Instant`, `SystemTime` |
| **Networking** | `TcpStream`, `TcpListener`, `UdpSocket`, `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`, `SocketAddrV6` |
| **Threading** | `Thread`, `JoinHandle<T>`, `Sender<T>`, `SyncSender<T>`, `Receiver<T>`, `Barrier`, `BarrierWaitResult` |
| **Atomics** | `AtomicBool`, `AtomicI8`–`AtomicI64`, `AtomicIsize`, `AtomicU8`–`AtomicU64`, `AtomicUsize` |
| **Numeric wrappers** | `NonZeroI8`–`NonZeroI128`, `NonZeroIsize`, `NonZeroU8`–`NonZeroU128`, `NonZeroUsize`, `Wrapping<T>`, `Saturating<T>` |
| **Tuples** | `(A,)` through `(A, B, C, D, E, F, G, H, I, J, K, L)` (1–12 elements) |
| **Arrays** | `[T; N]` (const generic, any size) |
| **Static slices** | `&'static [T]`, `&'static mut [T]` |

**Note on generic types**: For generic types like `Vec<T>`, the display symbol
does not include the type parameter (for example, `Vec<i32>` and `Vec<String>`
both display as `"Vec"`). `Any::downcast` still distinguishes the full concrete
types. Use a derived newtype when distinct R-facing names and clearer mismatch
diagnostics matter.

**Note on `ManuallyDrop<T>`**: It shares `T`'s display and diagnostic symbols,
but remains a distinct concrete type to `Any::downcast`. Matching symbols do
not make `ExternalPtr<ManuallyDrop<T>>` interchangeable with `ExternalPtr<T>`.

**Note on static slices**: `&'static [T]` and `&'static mut [T]` are fat pointers (ptr + len) that satisfy `'static + Sized`, so they can be stored directly in `ExternalPtr`. Use cases include const arrays (`&DATA`), leaked data (`Box::leak`), and memory-mapped files.

### IntoExternalPtr

The `IntoExternalPtr` marker trait triggers a blanket `IntoR` implementation that wraps the value in `ExternalPtr<T>` when returning from `#[miniextendr]` functions. `#[derive(ExternalPtr)]` implements both `TypedExternal` and `IntoExternalPtr`.

## Concrete Type Safety and Cross-Package Dispatch

`ExternalPtr<T>` stores `Box<Box<dyn Any>>`. `wrap_sexp()` asks that `Any`
value for `T` with `downcast_mut`; `wrap_sexp_with_error()` reports the stored
diagnostic name on failure. `TYPE_ID_CSTR` (`crate@version::module::Type`) makes
that message unambiguous, but does not decide whether the downcast succeeds.

Do not use matching R symbols as a cross-package concrete-type contract. For
cross-package behavior without a shared concrete Rust type, export a trait and
use the stable tag/vtable protocol described in [Trait ABI](TRAIT_ABI.md).

## Type-Erased Pointers (ErasedExternalPtr)

```rust
pub type ErasedExternalPtr = ExternalPtr<()>;
```

`ErasedExternalPtr` opens a miniextendr-created `EXTPTRSXP` without selecting a
concrete pointee type first. Its address must still contain miniextendr's
`Box<Box<dyn Any>>` representation; it is not a wrapper for arbitrary foreign
external pointers. It is useful for:

- Inspecting the stored type name before downcasting
- Inspecting miniextendr external pointers when the concrete Rust type is not
  known statically

```rust
let erased = unsafe { ErasedExternalPtr::from_sexp(some_sexp) };

// Check what type is stored
if erased.is::<MyData>() {
    let data: &MyData = erased.downcast_ref::<MyData>().unwrap();
}

// Or read the stored type name
if let Some(name) = erased.stored_type_name() {
    println!("stored type: {}", name);
}
```

Methods on `ErasedExternalPtr`:

| Method | Returns |
|--------|---------|
| `is::<T>()` | `bool` -- does the stored type match `T`? |
| `downcast_ref::<T>()` | `Option<&T>` |
| `downcast_mut::<T>()` | `Option<&mut T>` |
| `stored_type_name()` | `Option<&'static str>` |

## ExternalSlice

`ExternalSlice<T>` stores a `Vec<T>` as a raw pointer + length + capacity, suitable for wrapping in `ExternalPtr`:

```rust
impl_typed_external!(ExternalSlice<f64>);

let data = vec![1.0, 2.0, 3.0];
let ptr = ExternalPtr::new(ExternalSlice::new(data));
assert_eq!(ptr.as_slice(), &[1.0, 2.0, 3.0]);
```

This is useful when you need R to own a Rust slice and access it by index (e.g., in ALTREP `elt` callbacks).

| Method | Returns |
|--------|---------|
| `new(vec)` | Creates from `Vec<T>` |
| `from_boxed(boxed)` | Creates from `Box<[T]>` |
| `as_slice()` | `&[T]` |
| `as_mut_slice()` | `&mut [T]` |
| `len()` / `is_empty()` | Length queries |

## ALTREP data1/data2 Helpers

ALTREP objects have two data slots (`data1`, `data2`). These helpers extract typed `ExternalPtr`s from those slots:

```rust
// In an ALTREP callback:
fn length(x: SEXP) -> R_xlen_t {
    match unsafe { altrep_data1_as::<MyAltrepData>(x) } {
        Some(ext) => ext.data.len() as R_xlen_t,
        None => 0,
    }
}
```

| Function | Description |
|----------|-------------|
| `altrep_data1_as::<T>(x)` | Extract data1 as `ExternalPtr<T>` with type check |
| `altrep_data1_as_unchecked::<T>(x)` | Same, skips thread safety assertions |
| `altrep_data2_as::<T>(x)` | Extract data2 as `ExternalPtr<T>` with type check |
| `altrep_data2_as_unchecked::<T>(x)` | Same, skips thread safety assertions |
| `altrep_data1_mut::<T>(x)` | Mutable `&'static mut T` reference from data1 |
| `altrep_data1_mut_unchecked::<T>(x)` | Same, skips thread safety assertions |

The `_unchecked` variants are for performance-critical ALTREP callbacks where you are guaranteed to be on the main thread.

## RSidecar (R Data Fields)

`RSidecar` is a zero-sized marker type that enables R-facing getter/setter generation for struct fields annotated with `#[r_data]`:

```rust
#[derive(ExternalPtr)]
pub struct MyType {
    pub x: i32,

    #[r_data]
    r: RSidecar,          // Enables R wrapper generation

    #[r_data]
    pub count: i32,       // Generates MyType_get_count() / MyType_set_count()

    #[r_data]
    pub name: String,     // Generates MyType_get_name() / MyType_set_name()
}
```

Only `pub` fields with `#[r_data]` get R wrapper functions. Supported field types: `SEXP`, `i32`, `f64`, `bool`, `u8`, and any type implementing `IntoR`.

## SEXP Layout

The internal layout of an `ExternalPtr`-created `EXTPTRSXP`:

```text
EXTPTRSXP
  addr  → *mut Box<dyn Any> (thin pointer → heap-allocated fat pointer)
            └→ Box<T> (the actual Rust value)
  tag   → SYMSXP (TYPE_NAME_CSTR, for display)
  prot  → VECSXP[2]
            [0] → SYMSXP (TYPE_ID_CSTR, for mismatch diagnostics)
            [1] → user-protected SEXP (set via set_protected)
```

Internally the value is stored as `Box<Box<dyn Any>>`: the outer `Box` is a thin pointer that fits in R's `EXTPTRSXP` `addr` slot, and the inner `Box<dyn Any>` carries the trait-object vtable needed for `Any::downcast` at retrieval time. This lets one non-generic finalizer (`release_any`) free any `T` without per-type monomorphization. Type safety relies on `Any::downcast`, not on the `prot` symbols.

The `prot` slot holds a two-element list. Slot 0 is the namespaced type ID symbol, retained for display/debug parity; authoritative type checking is `Any::downcast`. Slot 1 is available for user-protected R objects that should be kept alive alongside the pointer.

## Thread Safety

`ExternalPtr` is `Send` when `T: Send`, allowing it to be transferred between threads. All R API calls are serialized through the main thread. Concurrent access is not supported -- R's runtime is single-threaded.

## Trait Implementations

`ExternalPtr<T>` mirrors `Box<T>`'s trait implementations:

- `Deref<Target = T>` / `DerefMut`
- `AsRef<T>` / `AsMut<T>` / `Borrow<T>` / `BorrowMut<T>`
- `Clone` (deep clone, when `T: Clone`)
- `Default` (when `T: Default`)
- `Debug` / `Display` / `Pointer`
- `PartialEq` / `Eq` / `PartialOrd` / `Ord` / `Hash` (compare pointee values)
- `Iterator` / `DoubleEndedIterator` / `ExactSizeIterator` / `FusedIterator`
- `From<T>` / `From<Box<T>>`
- `Pin` support via `pin()`, `pin_unchecked()`, `into_pin()`
