#include "test_wrapper.h"
#include <string>
#include <memory>
#include <sstream>
#include <iomanip>

// Include Longfellow headers
#include "algebra/fp_p128.h"
#include "algebra/fft.h"
#include "algebra/poly.h"
#include "arrays/dense.h"
#include "arrays/sparse.h"

// Simple JSON parsing (in production, use a proper JSON library)
#include <map>
#include <vector>

using namespace proofs;

struct CppTestContext {
    std::unique_ptr<Fp128<>> field;
    
    CppTestContext() : field(std::make_unique<Fp128<>>()) {}
};

extern "C" {

CppTestContext* create_test_context() {
    return new CppTestContext();
}

void destroy_test_context(CppTestContext* ctx) {
    delete ctx;
}

// Helper to convert field element to hex string
template<typename Field>
std::string field_to_hex(const typename Field::Elt& elem, const Field& F) {
    auto n = F.from_montgomery(elem);
    std::stringstream ss;
    for (int i = Field::N::kU64 - 1; i >= 0; --i) {
        ss << std::hex << std::setfill('0') << std::setw(16) << n.u64_[i];
    }
    return ss.str();
}

// Helper to parse field element from decimal string
template<typename Field>
typename Field::Elt parse_field_decimal(const std::string& s, const Field& F) {
    return F.of_scalar(std::stoull(s));
}

int run_field_arithmetic_test(CppTestContext* ctx, const char* input_json, char** output_json) {
    try {
        // Parse JSON (simplified - in production use proper JSON parser)
        std::string input(input_json);
        
        // Extract operation
        size_t op_pos = input.find("\"op\":\"") + 6;
        size_t op_end = input.find("\"", op_pos);
        std::string op = input.substr(op_pos, op_end - op_pos);
        
        // Extract a
        size_t a_pos = input.find("\"a\":\"") + 5;
        size_t a_end = input.find("\"", a_pos);
        std::string a_str = input.substr(a_pos, a_end - a_pos);
        auto a = parse_field_decimal(a_str, *ctx->field);
        
        typename Fp128<>::Elt result;
        
        if (op == "add" || op == "sub" || op == "mul") {
            // Extract b
            size_t b_pos = input.find("\"b\":\"") + 5;
            size_t b_end = input.find("\"", b_pos);
            std::string b_str = input.substr(b_pos, b_end - b_pos);
            auto b = parse_field_decimal(b_str, *ctx->field);
            
            if (op == "add") {
                result = ctx->field->addf(a, b);
            } else if (op == "sub") {
                result = ctx->field->subf(a, b);
            } else if (op == "mul") {
                result = ctx->field->mulf(a, b);
            }
        } else if (op == "neg") {
            result = ctx->field->negf(a);
        } else if (op == "inv") {
            result = ctx->field->invertf(a);
        }
        
        // Create output JSON
        std::string output = "{\"result\":\"" + field_to_hex(result, *ctx->field) + "\"}";
        
        *output_json = new char[output.length() + 1];
        strcpy(*output_json, output.c_str());
        
        return 0;
    } catch (const std::exception& e) {
        std::string error = std::string("Error: ") + e.what();
        *output_json = new char[error.length() + 1];
        strcpy(*output_json, error.c_str());
        return 1;
    }
}

int run_fft_test(CppTestContext* ctx, const char* input_json, char** output_json) {
    try {
        // Parse JSON (simplified)
        std::string input(input_json);
        
        // Extract size
        size_t size_pos = input.find("\"size\":") + 7;
        size_t size_end = input.find(",", size_pos);
        size_t size = std::stoull(input.substr(size_pos, size_end - size_pos));
        
        // Extract omega
        size_t omega_pos = input.find("\"omega\":\"") + 9;
        size_t omega_end = input.find("\"", omega_pos);
        std::string omega_str = input.substr(omega_pos, omega_end - omega_pos);
        auto omega = parse_field_decimal(omega_str, *ctx->field);
        
        // Extract coefficients
        std::vector<typename Fp128<>::Elt> coeffs;
        size_t coeffs_pos = input.find("\"coefficients\":[") + 16;
        size_t coeffs_end = input.find("]", coeffs_pos);
        std::string coeffs_str = input.substr(coeffs_pos, coeffs_end - coeffs_pos);
        
        size_t pos = 0;
        while (pos < coeffs_str.length()) {
            size_t quote_start = coeffs_str.find("\"", pos);
            if (quote_start == std::string::npos) break;
            size_t quote_end = coeffs_str.find("\"", quote_start + 1);
            std::string coeff_str = coeffs_str.substr(quote_start + 1, quote_end - quote_start - 1);
            coeffs.push_back(parse_field_decimal(coeff_str, *ctx->field));
            pos = quote_end + 1;
        }
        
        // Extract inverse flag
        bool inverse = input.find("\"inverse\":true") != std::string::npos;
        
        // Create FFT and run
        FFT<Fp128<>> fft(*ctx->field, size, omega);
        
        if (inverse) {
            fft.inverse(coeffs.data());
        } else {
            fft.forward(coeffs.data());
        }
        
        // Create output JSON
        std::string output = "{\"values\":[";
        for (size_t i = 0; i < coeffs.size(); ++i) {
            if (i > 0) output += ",";
            output += "\"" + field_to_hex(coeffs[i], *ctx->field) + "\"";
        }
        output += "]}";
        
        *output_json = new char[output.length() + 1];
        strcpy(*output_json, output.c_str());
        
        return 0;
    } catch (const std::exception& e) {
        std::string error = std::string("Error: ") + e.what();
        *output_json = new char[error.length() + 1];
        strcpy(*output_json, error.c_str());
        return 1;
    }
}

int run_polynomial_test(CppTestContext* ctx, const char* input_json, char** output_json) {
    // Implementation similar to above
    // Parse input, perform polynomial operations, return JSON output
    return 0;
}

int run_dense_array_test(CppTestContext* ctx, const char* input_json, char** output_json) {
    // Implementation similar to above
    // Parse input, perform dense array operations, return JSON output
    return 0;
}

int run_sparse_array_test(CppTestContext* ctx, const char* input_json, char** output_json) {
    // Implementation similar to above
    // Parse input, perform sparse array operations, return JSON output
    return 0;
}

void free_string(char* s) {
    delete[] s;
}

} // extern "C"