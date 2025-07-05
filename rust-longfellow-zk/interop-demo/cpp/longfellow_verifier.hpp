#ifndef LONGFELLOW_VERIFIER_HPP
#define LONGFELLOW_VERIFIER_HPP

#include <cstdint>
#include <string>
#include <vector>
#include <memory>

namespace longfellow {

// Forward declarations
struct ProofHandle;
struct FieldElement;

// Verification result
struct VerificationResult {
    bool valid;
    const char* error_message;
    bool ligero_valid;
    bool sumcheck_valid;
    uint64_t verification_time_ms;
};

// Field element representation matching Rust
struct Fp128 {
    uint64_t limbs[2];
    
    Fp128() : limbs{0, 0} {}
    Fp128(uint64_t value) : limbs{value, 0} {}
    
    static Fp128 from_bytes(const uint8_t* bytes, size_t len);
    std::vector<uint8_t> to_bytes() const;
};

// Main verifier class
class Verifier {
public:
    Verifier();
    ~Verifier();
    
    // Load proof from different formats
    bool load_proof_from_bytes(const uint8_t* data, size_t len);
    bool load_proof_from_json(const std::string& json);
    bool load_proof_from_file(const std::string& filename);
    
    // Verify the loaded proof
    VerificationResult verify();
    
    // Get proof metadata
    std::string get_metadata_json() const;
    
    // Batch verification
    static std::vector<VerificationResult> batch_verify(
        const std::vector<std::unique_ptr<Verifier>>& verifiers
    );
    
private:
    ProofHandle* proof_handle;
    
    // Disable copy
    Verifier(const Verifier&) = delete;
    Verifier& operator=(const Verifier&) = delete;
};

// Utility functions
namespace util {
    // Convert field element from u64
    Fp128 field_from_u64(uint64_t value);
    
    // Compute hash of data
    std::vector<uint8_t> sha256(const uint8_t* data, size_t len);
    
    // Base64 encoding/decoding
    std::string base64_encode(const uint8_t* data, size_t len);
    std::vector<uint8_t> base64_decode(const std::string& encoded);
}

// C API for FFI
extern "C" {
    // Opaque handle type
    typedef struct ProofHandle ProofHandle;
    
    // Field element type
    typedef struct {
        uint64_t limbs[2];
    } CFieldElement;
    
    // Verification result type
    typedef struct {
        bool valid;
        const char* error_message;
        bool ligero_valid;
        bool sumcheck_valid;
        uint64_t verification_time_ms;
    } CVerificationResult;
    
    // Create proof from bytes
    ProofHandle* longfellow_proof_from_bytes(const uint8_t* data, size_t len);
    
    // Create proof from JSON
    ProofHandle* longfellow_proof_from_json(const char* json_str);
    
    // Verify proof
    CVerificationResult longfellow_verify_proof(const ProofHandle* proof_handle);
    
    // Free proof handle
    void longfellow_proof_free(ProofHandle* handle);
    
    // Free error message
    void longfellow_error_free(char* error);
    
    // Get proof metadata as JSON
    char* longfellow_proof_metadata_json(const ProofHandle* proof_handle);
    
    // Field element operations
    CFieldElement longfellow_field_from_u64(uint64_t value);
    
    // Batch verification
    bool longfellow_batch_verify(
        const ProofHandle** proof_handles,
        size_t count,
        CVerificationResult* results
    );
}

} // namespace longfellow

#endif // LONGFELLOW_VERIFIER_HPP