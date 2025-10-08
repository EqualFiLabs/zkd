#pragma once

#include <stdint.h>
#include <stddef.h>

/*
 * Platform notes:
 *   - Linux and Android ship libzkprov as a shared object (libzkprov.so).
 *   - macOS ships libzkprov as a dynamic library (libzkprov.dylib).
 *   - Android distributions embed the .so inside the application package.
 */

#ifdef __cplusplus
extern "C" {
#endif

/* Error codes (see task 0.9.B). */
#define ZKP_OK 0
#define ZKP_ERR_INVALID_ARG 1
#define ZKP_ERR_BACKEND 2
#define ZKP_ERR_PROFILE 3
#define ZKP_ERR_PROOF_CORRUPT 4
#define ZKP_ERR_VERIFY_FAIL 5
#define ZKP_ERR_INTERNAL 6

/**
 * Initialize the prover runtime. This function is idempotent and does not
 * allocate memory on success. Returns ZKP_OK on success or an error code on
 * failure.
 */
int32_t zkp_init(void);

/**
 * Retrieve a JSON description of all registered backends.
 *
 * On success, *out_json receives a heap-allocated, NUL-terminated UTF-8 string
 * owned by the prover runtime. The caller must release any non-NULL pointer
 * stored in *out_json via zkp_free when it is no longer needed.
 */
int32_t zkp_list_backends(char **out_json);

/**
 * Retrieve a JSON description of the available proving profiles.
 *
 * On success, *out_json receives a heap-allocated, NUL-terminated UTF-8 string
 * owned by the prover runtime. The caller must release any non-NULL pointer
 * stored in *out_json via zkp_free when it is no longer needed.
 */
int32_t zkp_list_profiles(char **out_json);

/**
 * Generate a proof and metadata for the supplied AIR program.
 *
 * Parameters and ownership rules:
 *   - backend_id, field, hash_id, profile_id, air_path, and public_inputs_json
 *     must point to caller-owned, NUL-terminated UTF-8 strings.
 *   - On success, *out_proof receives a heap-allocated buffer containing the
 *     proof bytes and *out_proof_len receives its length in bytes. The caller
 *     owns *out_proof and must release any non-NULL value with zkp_free.
 *   - On success, *out_json_meta receives a heap-allocated, NUL-terminated
 *     UTF-8 string describing the proof metadata. The caller must release any
 *     non-NULL value with zkp_free.
 */
int32_t zkp_prove(
    const char *backend_id,
    const char *field,
    const char *hash_id,
    uint32_t fri_arity,
    const char *profile_id,
    const char *air_path,
    const char *public_inputs_json,
    uint8_t **out_proof,
    uint64_t *out_proof_len,
    char **out_json_meta
);

/**
 * Verify a proof previously produced by zkp_prove.
 *
 * Parameters and ownership rules mirror zkp_prove. The proof_ptr/proof_len pair
 * must reference caller-owned proof bytes. On success, *out_json_meta receives a
 * heap-allocated, NUL-terminated UTF-8 string that the caller must free with
 * zkp_free when finished.
 */
int32_t zkp_verify(
    const char *backend_id,
    const char *field,
    const char *hash_id,
    uint32_t fri_arity,
    const char *profile_id,
    const char *air_path,
    const char *public_inputs_json,
    const uint8_t *proof_ptr,
    uint64_t proof_len,
    char **out_json_meta
);

/**
 * Allocate a buffer owned by the prover runtime. Callers must eventually
 * release any non-NULL pointer returned from this function with zkp_free.
 */
void *zkp_alloc(uint64_t nbytes);

/**
 * Release memory previously allocated by the prover runtime. Passing NULL is a
 * no-op. Call this for every non-NULL pointer returned directly by the API or
 * written into an out-parameter by the API.
 */
void zkp_free(void *ptr);

#ifdef __cplusplus
}
#endif

