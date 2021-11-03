#include <linux/userfaultfd.h>
#ifndef UFFD_USER_MODE_ONLY
// this definition is not available before Linux 5.11. It is provided so
// userfaultfd-sys has the same exports on all kernels
#define UFFD_USER_MODE_ONLY 1
#endif
