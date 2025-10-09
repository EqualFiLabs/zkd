#include <napi.h>
#include "zkprov.h"

#include <cmath>
#include <limits>
#include <string>
#include <utility>

namespace {

const char *GetErrorMessage(int32_t code) {
  switch (code) {
    case ZKP_OK:
      return "ok";
    case ZKP_ERR_INVALID_ARG:
      return "Invalid argument";
    case ZKP_ERR_BACKEND:
      return "Backend error";
    case ZKP_ERR_PROFILE:
      return "Profile error";
    case ZKP_ERR_PROOF_CORRUPT:
      return "Proof corrupt";
    case ZKP_ERR_VERIFY_FAIL:
      return "Verification failed";
    case ZKP_ERR_INTERNAL:
    default:
      return "Internal error";
  }
}

Napi::Object CreateErrorObject(Napi::Env env, int32_t code, const std::string &detail,
                               const std::string &message = std::string()) {
  Napi::Object err = Napi::Object::New(env);
  err.Set("code", Napi::Number::New(env, code));
  const std::string msg = message.empty() ? std::string(GetErrorMessage(code)) : message;
  err.Set("msg", Napi::String::New(env, msg));
  if (!detail.empty()) {
    err.Set("detail", Napi::String::New(env, detail));
  }
  return err;
}

Napi::Object ParseJsonObjectOrFallback(Napi::Env env, const std::string &json) {
  if (json.empty()) {
    return Napi::Object::New(env);
  }

  Napi::String json_text = Napi::String::New(env, json);
  Napi::Value json_namespace = env.Global().Get("JSON");
  if (json_namespace.IsObject()) {
    Napi::Object json_object = json_namespace.As<Napi::Object>();
    Napi::Value parse_value = json_object.Get("parse");
    if (parse_value.IsFunction()) {
      Napi::Function parse = parse_value.As<Napi::Function>();
      Napi::Value parsed = parse.Call(json_object, {json_text});
      if (!env.IsExceptionPending() && parsed.IsObject()) {
        return parsed.As<Napi::Object>();
      }
    }
  }

  if (env.IsExceptionPending()) {
    env.GetAndClearPendingException();
  }

  Napi::Object fallback = Napi::Object::New(env);
  fallback.Set("raw", json_text);
  return fallback;
}

struct CommonConfig {
  std::string backend_id;
  std::string field;
  std::string hash_id;
  uint32_t fri_arity = 0;
  std::string profile_id;
  std::string air_path;
  std::string public_inputs_json;
};

bool GetStringProperty(const Napi::Object &obj, const char *key, bool allow_empty,
                       std::string *out_value, std::string *detail) {
  if (!obj.Has(key)) {
    *detail = std::string("Missing required property '") + key + "'";
    return false;
  }

  Napi::Value value = obj.Get(key);
  if (!value.IsString()) {
    *detail = std::string("Property '") + key + "' must be a string";
    return false;
  }

  std::string str = value.As<Napi::String>().Utf8Value();
  if (!allow_empty && str.empty()) {
    *detail = std::string("Property '") + key + "' must be a non-empty string";
    return false;
  }

  *out_value = std::move(str);
  return true;
}

bool ParseCommonConfig(const Napi::Value &value, CommonConfig *out_config, std::string *detail) {
  if (!value.IsObject()) {
    *detail = "Configuration must be an object";
    return false;
  }

  Napi::Object obj = value.As<Napi::Object>();
  if (!GetStringProperty(obj, "backendId", false, &out_config->backend_id, detail)) {
    return false;
  }
  if (!GetStringProperty(obj, "field", false, &out_config->field, detail)) {
    return false;
  }
  if (!GetStringProperty(obj, "hashId", false, &out_config->hash_id, detail)) {
    return false;
  }
  if (!GetStringProperty(obj, "profileId", false, &out_config->profile_id, detail)) {
    return false;
  }
  if (!GetStringProperty(obj, "airPath", false, &out_config->air_path, detail)) {
    return false;
  }
  if (!GetStringProperty(obj, "publicInputsJson", true, &out_config->public_inputs_json, detail)) {
    return false;
  }

  if (!obj.Has("friArity")) {
    *detail = "Missing required property 'friArity'";
    return false;
  }

  Napi::Value fri_value = obj.Get("friArity");
  if (!fri_value.IsNumber()) {
    *detail = "Property 'friArity' must be a positive integer";
    return false;
  }

  double fri_double = fri_value.As<Napi::Number>().DoubleValue();
  if (fri_double < 1.0 || fri_double > static_cast<double>(std::numeric_limits<uint32_t>::max()) ||
      std::floor(fri_double) != fri_double) {
    *detail = "Property 'friArity' must be a positive integer";
    return false;
  }

  out_config->fri_arity = fri_value.As<Napi::Number>().Uint32Value();
  return true;
}

class PromiseWorker : public Napi::AsyncWorker {
 public:
  explicit PromiseWorker(Napi::Env env)
      : Napi::AsyncWorker(env), deferred_(Napi::Promise::Deferred::New(env)) {}

  Napi::Promise GetPromise() { return deferred_.Promise(); }

 protected:
  void Fail(int32_t code, const std::string &detail, const std::string &message = std::string()) {
    error_code_ = code;
    error_detail_ = detail;
    error_message_ = message.empty() ? std::string(GetErrorMessage(code)) : message;
    SetError(error_message_);
  }

  void OnError(const Napi::Error & /*error*/) override {
    Napi::Env env = Env();
    Napi::Object err = CreateErrorObject(env, error_code_, error_detail_, error_message_);
    deferred_.Reject(err);
  }

  void Resolve(const Napi::Value &value) { deferred_.Resolve(value); }

 private:
  Napi::Promise::Deferred deferred_;
  int32_t error_code_ = ZKP_ERR_INTERNAL;
  std::string error_detail_;
  std::string error_message_;
};

class ProveWorker : public PromiseWorker {
 public:
  ProveWorker(Napi::Env env, CommonConfig config)
      : PromiseWorker(env), config_(std::move(config)) {}

  ~ProveWorker() override {
    if (proof_ptr_ != nullptr) {
      zkp_free(proof_ptr_);
      proof_ptr_ = nullptr;
    }
  }

 protected:
  void Execute() override {
    int32_t rc = zkp_init();
    if (rc != ZKP_OK) {
      Fail(rc, "zkp_init failed");
      return;
    }

    uint8_t *proof_ptr = nullptr;
    uint64_t proof_len = 0;
    char *meta_json = nullptr;

    rc = zkp_prove(config_.backend_id.c_str(), config_.field.c_str(), config_.hash_id.c_str(),
                   config_.fri_arity, config_.profile_id.c_str(), config_.air_path.c_str(),
                   config_.public_inputs_json.c_str(), &proof_ptr, &proof_len, &meta_json);
    if (rc != ZKP_OK) {
      if (proof_ptr != nullptr) {
        zkp_free(proof_ptr);
      }
      if (meta_json != nullptr) {
        zkp_free(meta_json);
      }
      Fail(rc, "zkp_prove failed");
      return;
    }

    proof_ptr_ = proof_ptr;
    proof_len_ = proof_len;
    if (meta_json != nullptr) {
      meta_json_ = meta_json;
      zkp_free(meta_json);
    }
  }

  void OnOK() override {
    Napi::Env env = Env();
    Napi::Object result = Napi::Object::New(env);

    if (proof_ptr_ != nullptr) {
      Napi::Buffer<uint8_t> proof_buffer = Napi::Buffer<uint8_t>::New(
          env, proof_ptr_, static_cast<size_t>(proof_len_),
          [](Napi::Env /*env*/, uint8_t *data) { zkp_free(data); });
      proof_ptr_ = nullptr;
      result.Set("proof", proof_buffer);
    } else {
      result.Set("proof", Napi::Buffer<uint8_t>::New(env, 0));
    }

    result.Set("meta", ParseJsonObjectOrFallback(env, meta_json_));
    Resolve(result);
  }

 private:
  CommonConfig config_;
  uint8_t *proof_ptr_ = nullptr;
  uint64_t proof_len_ = 0;
  std::string meta_json_;
};

class VerifyWorker : public PromiseWorker {
 public:
  VerifyWorker(Napi::Env env, CommonConfig config, Napi::Buffer<uint8_t> proof)
      : PromiseWorker(env),
        config_(std::move(config)),
        proof_ref_(Napi::Persistent(proof)),
        proof_ptr_(proof.Data()),
        proof_len_(static_cast<uint64_t>(proof.Length())) {}

 protected:
  void Execute() override {
    int32_t rc = zkp_init();
    if (rc != ZKP_OK) {
      Fail(rc, "zkp_init failed");
      return;
    }

    char *meta_json = nullptr;
    rc = zkp_verify(config_.backend_id.c_str(), config_.field.c_str(), config_.hash_id.c_str(),
                    config_.fri_arity, config_.profile_id.c_str(), config_.air_path.c_str(),
                    config_.public_inputs_json.c_str(), proof_ptr_, proof_len_, &meta_json);

    if (rc == ZKP_OK) {
      verified_ = true;
    } else if (rc == ZKP_ERR_VERIFY_FAIL) {
      verified_ = false;
    } else {
      if (meta_json != nullptr) {
        zkp_free(meta_json);
      }
      Fail(rc, "zkp_verify failed");
      return;
    }

    if (meta_json != nullptr) {
      meta_json_ = meta_json;
      zkp_free(meta_json);
    }
  }

  void OnOK() override {
    Napi::Env env = Env();
    Napi::Object result = Napi::Object::New(env);
    result.Set("verified", Napi::Boolean::New(env, verified_));
    result.Set("meta", ParseJsonObjectOrFallback(env, meta_json_));
    Resolve(result);
  }

 private:
  CommonConfig config_;
  Napi::Reference<Napi::Buffer<uint8_t>> proof_ref_;
  const uint8_t *proof_ptr_ = nullptr;
  uint64_t proof_len_ = 0;
  bool verified_ = false;
  std::string meta_json_;
};

Napi::Promise RejectInvalidArg(Napi::Env env, const std::string &detail) {
  Napi::Promise::Deferred deferred = Napi::Promise::Deferred::New(env);
  deferred.Reject(CreateErrorObject(env, ZKP_ERR_INVALID_ARG, detail));
  return deferred.Promise();
}

Napi::Promise HandleListCall(Napi::Env env, int32_t (*fn)(char **), const char *name) {
  Napi::Promise::Deferred deferred = Napi::Promise::Deferred::New(env);

  int32_t rc = zkp_init();
  if (rc != ZKP_OK) {
    deferred.Reject(CreateErrorObject(env, rc, std::string("zkp_init failed during ") + name));
    return deferred.Promise();
  }

  char *json = nullptr;
  rc = fn(&json);
  if (rc != ZKP_OK) {
    if (json != nullptr) {
      zkp_free(json);
    }
    deferred.Reject(CreateErrorObject(env, rc, std::string(name) + " failed"));
    return deferred.Promise();
  }

  std::string json_string = json != nullptr ? std::string(json) : std::string();
  if (json != nullptr) {
    zkp_free(json);
  }

  deferred.Resolve(ParseJsonObjectOrFallback(env, json_string));
  return deferred.Promise();
}

Napi::Value ListBackends(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  return HandleListCall(env, zkp_list_backends, "zkp_list_backends");
}

Napi::Value ListProfiles(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  return HandleListCall(env, zkp_list_profiles, "zkp_list_profiles");
}

Napi::Value Prove(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  if (info.Length() < 1) {
    return RejectInvalidArg(env, "Expected configuration object as first argument");
  }

  CommonConfig config;
  std::string detail;
  if (!ParseCommonConfig(info[0], &config, &detail)) {
    return RejectInvalidArg(env, detail);
  }

  ProveWorker *worker = new ProveWorker(env, std::move(config));
  Napi::Promise promise = worker->GetPromise();
  worker->Queue();
  return promise;
}

Napi::Value Verify(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  if (info.Length() < 2) {
    return RejectInvalidArg(env, "Expected configuration object and proof buffer");
  }

  CommonConfig config;
  std::string detail;
  if (!ParseCommonConfig(info[0], &config, &detail)) {
    return RejectInvalidArg(env, detail);
  }

  if (!info[1].IsBuffer()) {
    return RejectInvalidArg(env, "Proof must be a Buffer");
  }

  Napi::Buffer<uint8_t> proof = info[1].As<Napi::Buffer<uint8_t>>();
  VerifyWorker *worker = new VerifyWorker(env, std::move(config), proof);
  Napi::Promise promise = worker->GetPromise();
  worker->Queue();
  return promise;
}

}  // namespace

Napi::Object Init(Napi::Env env, Napi::Object exports) {
  exports.Set("listBackends", Napi::Function::New(env, ListBackends));
  exports.Set("listProfiles", Napi::Function::New(env, ListProfiles));
  exports.Set("prove", Napi::Function::New(env, Prove));
  exports.Set("verify", Napi::Function::New(env, Verify));
  return exports;
}

NODE_API_MODULE(zkprov, Init)
