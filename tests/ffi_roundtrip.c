#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "zkprov.h"

static void fail(const char *message, int32_t code) {
    if (code >= 0) {
        fprintf(stderr, "%s (code=%" PRId32 ")\n", message, (int32_t)code);
    } else {
        fprintf(stderr, "%s\n", message);
    }
    exit(EXIT_FAILURE);
}

static char *dup_range(const char *start, size_t len) {
    char *out = (char *)malloc(len + 1);
    if (!out) {
        return NULL;
    }
    memcpy(out, start, len);
    out[len] = '\0';
    return out;
}

static char *extract_digest(const char *json) {
    const char *pattern = "\"digest\":\"";
    const char *pos = strstr(json, pattern);
    if (!pos) {
        return NULL;
    }
    pos += strlen(pattern);
    const char *end = strchr(pos, '\"');
    if (!end) {
        return NULL;
    }
    return dup_range(pos, (size_t)(end - pos));
}

static int extract_verified_true(const char *json) {
    const char *pattern = "\"verified\":";
    const char *pos = strstr(json, pattern);
    if (!pos) {
        return 0;
    }
    pos += strlen(pattern);
    while (*pos == ' ' || *pos == '\t') {
        pos++;
    }
    return strncmp(pos, "true", 4) == 0;
}

static int extract_proof_len(const char *json, uint64_t *out_len) {
    const char *pattern = "\"proof_len\":";
    const char *pos = strstr(json, pattern);
    if (!pos) {
        return 0;
    }
    pos += strlen(pattern);
    while (*pos == ' ' || *pos == '\t') {
        pos++;
    }
    char *endptr = NULL;
    unsigned long long value = strtoull(pos, &endptr, 10);
    if (endptr == pos) {
        return 0;
    }
    if (out_len) {
        *out_len = (uint64_t)value;
    }
    return 1;
}

int main(void) {
    int32_t code = zkp_init();
    if (code != ZKP_OK) {
        fail("zkp_init failed", code);
    }

    char *backend_json = NULL;
    code = zkp_list_backends(&backend_json);
    if (code != ZKP_OK || backend_json == NULL) {
        fail("zkp_list_backends failed", code);
    }

    printf("Backends: %s\n", backend_json);
    if (strstr(backend_json, "\"id\":\"native@0.0\"") == NULL) {
        fprintf(stderr, "Expected native@0.0 backend in list\n");
        zkp_free(backend_json);
        return EXIT_FAILURE;
    }
    zkp_free(backend_json);

    const char *backend_id = "native@0.0";
    const char *field = "Prime254";
    const char *hash_id = "blake3";
    const uint32_t fri_arity = 2;
    const char *profile_id = "balanced";
    const char *air_path = "examples/air/toy.air";
    const char *public_inputs_json = "{\"demo\":true,\"n\":7}";

    uint8_t *proof = NULL;
    uint64_t proof_len = 0;
    char *prove_meta = NULL;
    code = zkp_prove(
        backend_id,
        field,
        hash_id,
        fri_arity,
        profile_id,
        air_path,
        public_inputs_json,
        &proof,
        &proof_len,
        &prove_meta
    );
    if (code != ZKP_OK) {
        if (prove_meta) {
            zkp_free(prove_meta);
        }
        if (proof) {
            zkp_free(proof);
        }
        fail("zkp_prove failed", code);
    }
    if (proof == NULL || proof_len == 0 || prove_meta == NULL) {
        if (prove_meta) {
            zkp_free(prove_meta);
        }
        if (proof) {
            zkp_free(proof);
        }
        fail("zkp_prove returned invalid outputs", -1);
    }

    uint64_t meta_proof_len = 0;
    if (!extract_proof_len(prove_meta, &meta_proof_len) || meta_proof_len != proof_len) {
        zkp_free(prove_meta);
        zkp_free(proof);
        fail("metadata proof_len mismatch", -1);
    }

    char *digest = extract_digest(prove_meta);
    if (digest == NULL) {
        zkp_free(prove_meta);
        zkp_free(proof);
        fail("metadata missing digest", -1);
    }

    zkp_free(prove_meta);

    char *verify_meta = NULL;
    code = zkp_verify(
        backend_id,
        field,
        hash_id,
        fri_arity,
        profile_id,
        air_path,
        public_inputs_json,
        proof,
        proof_len,
        &verify_meta
    );
    if (code != ZKP_OK || verify_meta == NULL) {
        free(digest);
        if (verify_meta) {
            zkp_free(verify_meta);
        }
        zkp_free(proof);
        fail("zkp_verify failed", code);
    }

    if (!extract_verified_true(verify_meta)) {
        free(digest);
        zkp_free(verify_meta);
        zkp_free(proof);
        fail("verification metadata missing verified=true", -1);
    }

    char *verify_digest = extract_digest(verify_meta);
    if (verify_digest == NULL || strcmp(verify_digest, digest) != 0) {
        free(digest);
        if (verify_digest) {
            free(verify_digest);
        }
        zkp_free(verify_meta);
        zkp_free(proof);
        fail("verification digest mismatch", -1);
    }

    printf("Verified: true\n");
    printf("Digest D: %s\n", digest);

    free(verify_digest);
    free(digest);
    zkp_free(verify_meta);
    zkp_free(proof);

    return EXIT_SUCCESS;
}
