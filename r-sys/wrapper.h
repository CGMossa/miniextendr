#define R_NO_REMAP
#define STRICT_R_HEADERS
#include <R.h>
#include <Rinternals.h>

//TODO: don't include Rdefines.h ever.

// Included by R.h:
//
// Rconfig.h	configuration info that is made available
// R_ext/Arith.h	handling for NAs, NaNs, Inf/-Inf
// R_ext/Boolean.h	TRUE/FALSE type
// R_ext/Complex.h	C typedefs for R’s complex
// R_ext/Constants.h	constants
// R_ext/Error.h	error signaling
// R_ext/Memory.h	memory allocation
// R_ext/Print.h	Rprintf and variations.
// R_ext/RS.h	definitions common to R.h and the former S.h, including F77_CALL etc.
// R_ext/Random.h	random number generation
// R_ext/Utils.h	sorting and other utilities
// [x] R_ext/libextern.h	definitions for exports from R.dll on Windows.

// R.h	includes many other files
// Rinternals.h	definitions for using R’s internal structures
// [x] Rdefines.h	macros for an S-like interface to the above (no longer maintained)
#define R_NO_REMAP_RMATH
// Rmath.h	standalone math library
// Rversion.h	R version information
// Rinterface.h	for add-on front-ends (Unix-alikes only)
// Rembedded.h	for add-on front-ends
// R_ext/Applic.h	optimization and integration
// R_ext/BLAS.h	C definitions for BLAS routines
// R_ext/Callbacks.h	C (and R function) top-level task handlers
// R_ext/GetX11Image.h	X11Image interface used by package trkplot
// R_ext/Lapack.h	C definitions for some LAPACK routines
// R_ext/Linpack.h	C definitions for some LINPACK routines, not all of which are included in R
// R_ext/Parse.h	a small part of R’s parse interface: not part of the stable API.
// R_ext/RStartup.h	for add-on front-ends
// R_ext/Rdynload.h	needed to register compiled code in packages
// R_ext/Riconv.h	interface to iconv
// R_ext/Visibility.h	definitions controlling visibility
// R_ext/eventloop.h	for add-on front-ends and for packages that need to share in the R event loops (not Windows)
