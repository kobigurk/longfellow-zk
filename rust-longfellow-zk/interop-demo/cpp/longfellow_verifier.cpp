#include "longfellow_verifier.hpp"
#include <fstream>
#include <sstream>
#include <cstring>
#include <algorithm>
#include <openssl/sha.h>
#include <openssl/evp.h>

namespace longfellow {

// Fp128 implementation
Fp128 Fp128::from_bytes(const uint8_t* bytes, size_t len) {
    Fp128 result;
    if (len >= 8) {
        std::memcpy(&result.limbs[0], bytes, 8);
    }
    if (len >= 16) {
        std::memcpy(&result.limbs[1], bytes + 8, 8);
    }
    return result;
}

std::vector<uint8_t> Fp128::to_bytes() const {
    std::vector<uint8_t> result(16);
    std::memcpy(result.data(), &limbs[0], 8);
    std::memcpy(result.data() + 8, &limbs[1], 8);
    return result;
}

// Verifier implementation
Verifier::Verifier() : proof_handle(nullptr) {}

Verifier::~Verifier() {
    if (proof_handle) {
        longfellow_proof_free(proof_handle);
    }
}

bool Verifier::load_proof_from_bytes(const uint8_t* data, size_t len) {
    if (proof_handle) {
        longfellow_proof_free(proof_handle);
    }
    
    proof_handle = longfellow_proof_from_bytes(data, len);
    return proof_handle != nullptr;
}

bool Verifier::load_proof_from_json(const std::string& json) {
    if (proof_handle) {
        longfellow_proof_free(proof_handle);
    }
    
    proof_handle = longfellow_proof_from_json(json.c_str());
    return proof_handle != nullptr;
}

bool Verifier::load_proof_from_file(const std::string& filename) {
    std::ifstream file(filename, std::ios::binary);
    if (!file) {
        return false;
    }
    
    // Check if JSON or binary
    if (filename.find(".json") != std::string::npos) {
        std::stringstream buffer;
        buffer << file.rdbuf();
        return load_proof_from_json(buffer.str());
    } else {
        // Binary format
        file.seekg(0, std::ios::end);
        size_t size = file.tellg();
        file.seekg(0, std::ios::beg);
        
        std::vector<uint8_t> data(size);
        file.read(reinterpret_cast<char*>(data.data()), size);
        
        return load_proof_from_bytes(data.data(), data.size());
    }
}

VerificationResult Verifier::verify() {
    if (!proof_handle) {
        return VerificationResult{
            false,
            "No proof loaded",
            false,
            false,
            0
        };
    }
    
    CVerificationResult c_result = longfellow_verify_proof(proof_handle);
    
    return VerificationResult{
        c_result.valid,
        c_result.error_message,
        c_result.ligero_valid,
        c_result.sumcheck_valid,
        c_result.verification_time_ms
    };
}

std::string Verifier::get_metadata_json() const {
    if (!proof_handle) {
        return "{}";
    }
    
    char* json_cstr = longfellow_proof_metadata_json(proof_handle);
    if (!json_cstr) {
        return "{}";
    }
    
    std::string result(json_cstr);
    longfellow_error_free(json_cstr);
    return result;
}

std::vector<VerificationResult> Verifier::batch_verify(
    const std::vector<std::unique_ptr<Verifier>>& verifiers
) {
    std::vector<const ProofHandle*> handles;
    handles.reserve(verifiers.size());
    
    for (const auto& v : verifiers) {
        handles.push_back(v->proof_handle);
    }
    
    std::vector<CVerificationResult> c_results(verifiers.size());
    
    bool success = longfellow_batch_verify(
        handles.data(),
        handles.size(),
        c_results.data()
    );
    
    std::vector<VerificationResult> results;
    results.reserve(verifiers.size());
    
    if (success) {
        for (const auto& c_result : c_results) {
            results.push_back(VerificationResult{
                c_result.valid,
                c_result.error_message,
                c_result.ligero_valid,
                c_result.sumcheck_valid,
                c_result.verification_time_ms
            });
        }
    }
    
    return results;
}

// Utility functions
namespace util {

Fp128 field_from_u64(uint64_t value) {
    CFieldElement c_elem = longfellow_field_from_u64(value);
    Fp128 result;
    result.limbs[0] = c_elem.limbs[0];
    result.limbs[1] = c_elem.limbs[1];
    return result;
}

std::vector<uint8_t> sha256(const uint8_t* data, size_t len) {
    std::vector<uint8_t> hash(SHA256_DIGEST_LENGTH);
    SHA256(data, len, hash.data());
    return hash;
}

std::string base64_encode(const uint8_t* data, size_t len) {
    // Calculate output length
    size_t out_len = 4 * ((len + 2) / 3);
    std::string result(out_len, '\0');
    
    // Use OpenSSL for encoding
    EVP_EncodeBlock(
        reinterpret_cast<unsigned char*>(&result[0]),
        data,
        len
    );
    
    // Remove padding if necessary
    result.erase(result.find_last_not_of('=') + 1);
    
    return result;
}

std::vector<uint8_t> base64_decode(const std::string& encoded) {
    // Calculate maximum output length
    size_t max_len = 3 * encoded.length() / 4;
    std::vector<uint8_t> result(max_len);
    
    // Decode
    int actual_len = EVP_DecodeBlock(
        result.data(),
        reinterpret_cast<const unsigned char*>(encoded.c_str()),
        encoded.length()
    );
    
    if (actual_len < 0) {
        return std::vector<uint8_t>();
    }
    
    // Adjust for padding
    int padding = 0;
    if (encoded.length() >= 2) {
        if (encoded[encoded.length() - 1] == '=') padding++;
        if (encoded[encoded.length() - 2] == '=') padding++;
    }
    
    result.resize(actual_len - padding);
    return result;
}

} // namespace util
} // namespace longfellow