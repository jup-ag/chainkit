// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// The following structs are used to implement the lowest level
// of the FFI, and thus useful to multiple uniffied crates.
// We ensure they are declared exactly once, with a header guard, UNIFFI_SHARED_H.
#ifdef UNIFFI_SHARED_H
    // We also try to prevent mixing versions of shared uniffi header structs.
    // If you add anything to the #else block, you must increment the version suffix in UNIFFI_SHARED_HEADER_V4
    #ifndef UNIFFI_SHARED_HEADER_V4
        #error Combining helper code from multiple versions of uniffi is not supported
    #endif // ndef UNIFFI_SHARED_HEADER_V4
#else
#define UNIFFI_SHARED_H
#define UNIFFI_SHARED_HEADER_V4
// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️

typedef struct RustBuffer
{
    int32_t capacity;
    int32_t len;
    uint8_t *_Nullable data;
} RustBuffer;

typedef int32_t (*ForeignCallback)(uint64_t, int32_t, const uint8_t *_Nonnull, int32_t, RustBuffer *_Nonnull);

typedef struct ForeignBytes
{
    int32_t len;
    const uint8_t *_Nullable data;
} ForeignBytes;

// Error definitions
typedef struct RustCallStatus {
    int8_t code;
    RustBuffer errorBuf;
} RustCallStatus;

// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️
#endif // def UNIFFI_SHARED_H

// Continuation callback for UniFFI Futures
typedef void (*UniFfiRustFutureContinuation)(void * _Nonnull, int8_t);

// Scaffolding functions
RustBuffer uniffi_chainkit_fn_func_append_signature_to_transaction(RustBuffer signer, RustBuffer signature, RustBuffer transaction, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_decrypt_ciphertext(RustBuffer ciphertext, RustBuffer password, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_derive(RustBuffer chain, RustBuffer mnemonic, RustBuffer passphrase, RustBuffer derivation, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_derive_from_data(RustBuffer chain, RustBuffer data, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_encrypt_plaintext(RustBuffer plaintext, RustBuffer password, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_generate_mnemonic(uint32_t length, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_get_associated_token_address(RustBuffer wallet_address, RustBuffer owner_program, RustBuffer token_mint_address, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_get_message(RustBuffer transaction, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_get_program_address(RustBuffer seeds, RustBuffer program, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_chainkit_fn_func_is_valid(RustBuffer chain, RustBuffer address, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_modify_transaction(RustBuffer chain, RustBuffer transaction, RustBuffer owner, RustBuffer parameters, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_parse_private_key(RustBuffer key, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_parse_public_key(RustBuffer address, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_parse_transaction(RustBuffer chain, RustBuffer transaction, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_raw_private_key(RustBuffer chain, RustBuffer key, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_send_transaction(RustBuffer chain, RustBuffer sender, RustBuffer receiver, RustBuffer amount, RustBuffer parameters, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_sign_message(RustBuffer chain, RustBuffer message, RustBuffer signers, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_sign_transaction(RustBuffer chain, RustBuffer transaction, RustBuffer signers, RustBuffer parameters, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_sign_typed_data(RustBuffer chain, RustBuffer typed_data, RustBuffer signers, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_chainkit_fn_func_token_transaction(RustBuffer chain, RustBuffer destination, RustBuffer owner, RustBuffer token, RustBuffer kind, RustBuffer parameters, RustCallStatus *_Nonnull out_status
);
RustBuffer ffi_chainkit_rustbuffer_alloc(int32_t size, RustCallStatus *_Nonnull out_status
);
RustBuffer ffi_chainkit_rustbuffer_from_bytes(ForeignBytes bytes, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rustbuffer_free(RustBuffer buf, RustCallStatus *_Nonnull out_status
);
RustBuffer ffi_chainkit_rustbuffer_reserve(RustBuffer buf, int32_t additional, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_u8(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_u8(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_u8(void* _Nonnull handle
);
uint8_t ffi_chainkit_rust_future_complete_u8(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_i8(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_i8(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_i8(void* _Nonnull handle
);
int8_t ffi_chainkit_rust_future_complete_i8(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_u16(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_u16(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_u16(void* _Nonnull handle
);
uint16_t ffi_chainkit_rust_future_complete_u16(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_i16(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_i16(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_i16(void* _Nonnull handle
);
int16_t ffi_chainkit_rust_future_complete_i16(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_u32(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_u32(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_u32(void* _Nonnull handle
);
uint32_t ffi_chainkit_rust_future_complete_u32(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_i32(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_i32(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_i32(void* _Nonnull handle
);
int32_t ffi_chainkit_rust_future_complete_i32(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_u64(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_u64(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_u64(void* _Nonnull handle
);
uint64_t ffi_chainkit_rust_future_complete_u64(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_i64(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_i64(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_i64(void* _Nonnull handle
);
int64_t ffi_chainkit_rust_future_complete_i64(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_f32(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_f32(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_f32(void* _Nonnull handle
);
float ffi_chainkit_rust_future_complete_f32(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_f64(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_f64(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_f64(void* _Nonnull handle
);
double ffi_chainkit_rust_future_complete_f64(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_pointer(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_pointer(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_pointer(void* _Nonnull handle
);
void*_Nonnull ffi_chainkit_rust_future_complete_pointer(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_rust_buffer(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_rust_buffer(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_rust_buffer(void* _Nonnull handle
);
RustBuffer ffi_chainkit_rust_future_complete_rust_buffer(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
void ffi_chainkit_rust_future_poll_void(void* _Nonnull handle, UniFfiRustFutureContinuation _Nonnull callback, void* _Nonnull callback_data
);
void ffi_chainkit_rust_future_cancel_void(void* _Nonnull handle
);
void ffi_chainkit_rust_future_free_void(void* _Nonnull handle
);
void ffi_chainkit_rust_future_complete_void(void* _Nonnull handle, RustCallStatus *_Nonnull out_status
);
uint16_t uniffi_chainkit_checksum_func_append_signature_to_transaction(void
    
);
uint16_t uniffi_chainkit_checksum_func_decrypt_ciphertext(void
    
);
uint16_t uniffi_chainkit_checksum_func_derive(void
    
);
uint16_t uniffi_chainkit_checksum_func_derive_from_data(void
    
);
uint16_t uniffi_chainkit_checksum_func_encrypt_plaintext(void
    
);
uint16_t uniffi_chainkit_checksum_func_generate_mnemonic(void
    
);
uint16_t uniffi_chainkit_checksum_func_get_associated_token_address(void
    
);
uint16_t uniffi_chainkit_checksum_func_get_message(void
    
);
uint16_t uniffi_chainkit_checksum_func_get_program_address(void
    
);
uint16_t uniffi_chainkit_checksum_func_is_valid(void
    
);
uint16_t uniffi_chainkit_checksum_func_modify_transaction(void
    
);
uint16_t uniffi_chainkit_checksum_func_parse_private_key(void
    
);
uint16_t uniffi_chainkit_checksum_func_parse_public_key(void
    
);
uint16_t uniffi_chainkit_checksum_func_parse_transaction(void
    
);
uint16_t uniffi_chainkit_checksum_func_raw_private_key(void
    
);
uint16_t uniffi_chainkit_checksum_func_send_transaction(void
    
);
uint16_t uniffi_chainkit_checksum_func_sign_message(void
    
);
uint16_t uniffi_chainkit_checksum_func_sign_transaction(void
    
);
uint16_t uniffi_chainkit_checksum_func_sign_typed_data(void
    
);
uint16_t uniffi_chainkit_checksum_func_token_transaction(void
    
);
uint32_t ffi_chainkit_uniffi_contract_version(void
    
);

