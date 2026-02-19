// Windows compatibility shim for unistd.h
// This provides minimal POSIX compatibility for zlib's gzip support on Windows MSVC
//
// zlib's gz*.c files include gzguts.h which unconditionally includes <unistd.h>
// On Windows MSVC, we provide this header to map POSIX I/O functions to their Windows equivalents

#ifndef UNISTD_H_WIN_COMPAT
#define UNISTD_H_WIN_COMPAT

#ifdef _WIN32

// Windows already has these I/O functions with underscore prefix in <io.h>
// We just need to include the necessary headers
#include <io.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>

// Map POSIX function names to Windows equivalents (if not already mapped)
// Note: gzguts.h already does some of this mapping via WINAPI_FAMILY check,
// but we provide fallbacks for standard MSVC builds

#ifndef open
#define open _open
#endif

#ifndef read  
#define read _read
#endif

#ifndef write
#define write _write
#endif

#ifndef close
#define close _close
#endif

#ifndef lseek
#define lseek _lseek
#endif

// For 64-bit file operations (already handled in gzlib.c via _lseeki64)
// Just ensure types are available
#ifndef off_t
typedef long off_t;
#endif

#ifndef ssize_t
typedef intptr_t ssize_t;
#endif

// Standard file descriptors
#ifndef STDIN_FILENO
#define STDIN_FILENO  0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2
#endif

// File access mode flags (from fcntl.h on Unix)
// These are typically defined in fcntl.h which we've included, but provide fallbacks
#ifndef F_OK
#define F_OK 0  // File exists
#define X_OK 1  // Execute permission
#define W_OK 2  // Write permission  
#define R_OK 4  // Read permission
#endif

#endif // _WIN32

#endif // UNISTD_H_WIN_COMPAT
