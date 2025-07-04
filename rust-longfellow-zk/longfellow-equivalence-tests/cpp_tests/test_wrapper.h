#ifndef LONGFELLOW_TEST_WRAPPER_H
#define LONGFELLOW_TEST_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

struct CppTestContext;

CppTestContext* create_test_context();
void destroy_test_context(CppTestContext* ctx);

int run_fft_test(CppTestContext* ctx, const char* input_json, char** output_json);
int run_field_arithmetic_test(CppTestContext* ctx, const char* input_json, char** output_json);
int run_polynomial_test(CppTestContext* ctx, const char* input_json, char** output_json);
int run_dense_array_test(CppTestContext* ctx, const char* input_json, char** output_json);
int run_sparse_array_test(CppTestContext* ctx, const char* input_json, char** output_json);

void free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif // LONGFELLOW_TEST_WRAPPER_H