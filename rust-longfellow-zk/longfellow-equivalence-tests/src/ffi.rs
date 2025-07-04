use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

#[repr(C)]
pub struct CppTestContext {
    _private: [u8; 0],
}

extern "C" {
    pub fn create_test_context() -> *mut CppTestContext;
    pub fn destroy_test_context(ctx: *mut CppTestContext);
    
    pub fn run_fft_test(
        ctx: *mut CppTestContext,
        input_json: *const c_char,
        output_json: *mut *mut c_char,
    ) -> i32;
    
    pub fn run_field_arithmetic_test(
        ctx: *mut CppTestContext,
        input_json: *const c_char,
        output_json: *mut *mut c_char,
    ) -> i32;
    
    pub fn run_polynomial_test(
        ctx: *mut CppTestContext,
        input_json: *const c_char,
        output_json: *mut *mut c_char,
    ) -> i32;
    
    pub fn run_dense_array_test(
        ctx: *mut CppTestContext,
        input_json: *const c_char,
        output_json: *mut *mut c_char,
    ) -> i32;
    
    pub fn run_sparse_array_test(
        ctx: *mut CppTestContext,
        input_json: *const c_char,
        output_json: *mut *mut c_char,
    ) -> i32;
    
    pub fn free_string(s: *mut c_char);
}

pub struct TestContext {
    ptr: *mut CppTestContext,
}

impl TestContext {
    pub fn new() -> Self {
        unsafe {
            Self {
                ptr: create_test_context(),
            }
        }
    }

    pub fn run_test<F>(&self, test_fn: F, input_json: &str) -> Result<String, String>
    where
        F: Fn(*mut CppTestContext, *const c_char, *mut *mut c_char) -> i32,
    {
        let c_input = CString::new(input_json).map_err(|e| e.to_string())?;
        let mut output_ptr: *mut c_char = std::ptr::null_mut();

        unsafe {
            let result = test_fn(self.ptr, c_input.as_ptr(), &mut output_ptr);
            
            if result != 0 {
                if !output_ptr.is_null() {
                    let error = CStr::from_ptr(output_ptr)
                        .to_string_lossy()
                        .into_owned();
                    free_string(output_ptr);
                    return Err(error);
                } else {
                    return Err(format!("Test failed with code {}", result));
                }
            }

            if output_ptr.is_null() {
                return Err("No output from test".to_string());
            }

            let output = CStr::from_ptr(output_ptr)
                .to_string_lossy()
                .into_owned();
            free_string(output_ptr);
            
            Ok(output)
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                destroy_test_context(self.ptr);
            }
        }
    }
}

unsafe impl Send for TestContext {}
unsafe impl Sync for TestContext {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = TestContext::new();
        assert!(!ctx.ptr.is_null());
    }
}