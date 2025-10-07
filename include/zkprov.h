#pragma once
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
  ZKP_OK = 0
} zkp_status_t;

// Initialize the library (idempotent).
zkp_status_t zkp_init(void);

// Free heap-allocated buffers returned by the library.
void zkp_free(void* ptr);

#ifdef __cplusplus
}
#endif
