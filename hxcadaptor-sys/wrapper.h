//#include <stdio.h>
#include <stddef.h>
struct _IO_FILE;
typedef struct _IO_FILE FILE;

// cpy pasted stuff needed for bindgen
#ifndef _W64
#  if !defined(__midl) && (defined(_X86_) || defined(_M_IX86)) && _MSC_VER >= 1300
#     define _W64 __w64
#  else
#     define _W64
#  endif
#endif


#ifndef _HXCFE_
typedef void HXCFE;
#define _HXCFE_
#endif

#ifndef _HXCFE_FLOPPY_
typedef void HXCFE_FLOPPY;
#define _HXCFE_FLOPPY_
#endif

#ifndef _HXCFE_SIDE_
typedef void HXCFE_SIDE;
#define _HXCFE_SIDE_
#endif

#ifndef _HXCFE_IMGLDR_
typedef void HXCFE_IMGLDR;
#define _HXCFE_IMGLDR_
#endif

#ifndef _HXCFE_XMLLDR_
typedef void HXCFE_XMLLDR;
#define _HXCFE_XMLLDR_
#endif

#ifndef _HXCFE_TD_
typedef void HXCFE_TD;
#define _HXCFE_TD_
#endif

#ifndef _HXCFE_TRKSTREAM_
typedef void HXCFE_TRKSTREAM;
#define _HXCFE_TRKSTREAM_
#endif

#ifndef _HXCFE_FXSA_
typedef void HXCFE_FXSA;
#define _HXCFE_FXSA_
#endif

#ifndef _HXCFE_FLPGEN_
typedef void HXCFE_FLPGEN;
#define _HXCFE_FLPGEN_
#endif

#ifndef _HXCFE_SECTORACCESS_
typedef void HXCFE_SECTORACCESS;
#define _HXCFE_SECTORACCESS_
#endif

#ifndef _HXCFE_FSMNG_
typedef void HXCFE_FSMNG;
#define _HXCFE_FSMNG_
#endif

#ifndef _HXCFE_FDCCTRL_
typedef void HXCFE_FDCCTRL;
#define _HXCFE_FDCCTRL_
#endif

#ifndef _HXCFE_SECTCFG_
typedef void HXCFE_SECTCFG;
#define _HXCFE_SECTCFG_
#endif


#ifdef _WIN64 // [
   typedef signed __int64    intptr_t;
   typedef unsigned __int64  uintptr_t;
#else // _WIN64 ][
   typedef _W64 signed int   intptr_t;
   typedef _W64 unsigned int uintptr_t;
#endif // _WIN64 ]

// file of interest
#include "libhxcadaptor.h"