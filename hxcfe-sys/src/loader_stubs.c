// Stub implementations for loaders that are excluded from the build
// but are still referenced in loaders_list.c

#include <stddef.h>

// Stub for ADZ loader (excluded because it needs gzip)
void* ADZ_libGetPluginInfo(void) {
    return NULL;
}

// Stub for IMZ loader (excluded because it needs minizip)
void* IMZ_libGetPluginInfo(void) {
    return NULL;
}

// Stubs for xdms functions used by DMS loader (xdms excluded)
int HXC_fopen(void* f, const char* path) {
    return -1;
}

void HXC_fclose(void* f) {
}

int Process_File(void* in, void* out, int cmd, int opt, int packcmd, int  packopt, int type) {
    return -1;
}
