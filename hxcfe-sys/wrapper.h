#include "types.h"
#include "internal_floppy.h"
#include "internal_libhxcfe.h"
#include "tracks/track_generator.h"
#include "libhxcfe.h"

// libhxcadaptor functions (requires stdio.h for FILE type)
#include <stdio.h>
#include "libhxcadaptor.h"

// libusbhxcfe functions (conditionally included)
#ifdef ENABLE_USB
#include "usb_hxcfloppyemulator.h"
#endif
