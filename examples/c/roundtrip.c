#include <stdio.h>
#include <stdint.h>
#include "zkprov.h"

int main() {
  if (zkp_init() != 0) {
    fprintf(stderr, "init fail\n");
    return 1;
  }

  char *json = NULL;
  zkp_list_backends(&json);
  printf("backends: %s\n", json);
  zkp_free(json);

  const char *backend = "native@0.0";
  const char *field = "Prime254";
  const char *hash = "blake3";
  const char *profile = "balanced";
  const char *air = "examples/air/toy.air";
  const char *inputs = "{\"demo\":true,\"n\":7}";
  uint8_t *proof = NULL;
  uint64_t plen = 0;
  char *meta = NULL;

  int rc = zkp_prove(backend, field, hash, 2, profile, air, inputs, &proof, &plen, &meta);
  if (rc) {
    fprintf(stderr, "prove err: %s\n", meta);
    zkp_free(meta);
    return rc;
  }

  printf("D=%s len=%llu\n", meta, (unsigned long long)plen);
  zkp_free(meta);

  meta = NULL;
  rc = zkp_verify(backend, field, hash, 2, profile, air, inputs, proof, plen, &meta);
  printf("verified=%s D=%s\n", rc == 0 ? "true" : "false", meta);
  zkp_free(meta);
  zkp_free(proof);
  return rc;
}
