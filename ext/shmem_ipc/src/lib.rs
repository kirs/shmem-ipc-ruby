use rb_sys::*;
use shmem_ipc::sharedring::{Receiver, Sender};
use std::ffi::{c_int, c_long, c_ulong, c_void};
use std::fs::File;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::ptr;

// Ruby class definitions
static mut SHMEM_IPC_MODULE: VALUE = Qnil as VALUE;
static mut FLOAT_SENDER_CLASS: VALUE = Qnil as VALUE;
static mut FLOAT_RECEIVER_CLASS: VALUE = Qnil as VALUE;
static mut INTEGER_SENDER_CLASS: VALUE = Qnil as VALUE;
static mut INTEGER_RECEIVER_CLASS: VALUE = Qnil as VALUE;
static mut BYTE_SENDER_CLASS: VALUE = Qnil as VALUE;
static mut BYTE_RECEIVER_CLASS: VALUE = Qnil as VALUE;

// Wrapper structs for different types
struct FloatSenderWrapper {
    sender: Option<Sender<f64>>,
}

struct FloatReceiverWrapper {
    receiver: Option<Receiver<f64>>,
}

struct IntegerSenderWrapper {
    sender: Option<Sender<i64>>,
}

struct IntegerReceiverWrapper {
    receiver: Option<Receiver<i64>>,
}

struct ByteSenderWrapper {
    sender: Option<Sender<u8>>,
}

struct ByteReceiverWrapper {
    receiver: Option<Receiver<u8>>,
}

// Free functions for Ruby GC
unsafe extern "C" fn float_sender_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut FloatSenderWrapper);
    }
}

unsafe extern "C" fn float_receiver_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut FloatReceiverWrapper);
    }
}

unsafe extern "C" fn integer_sender_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut IntegerSenderWrapper);
    }
}

unsafe extern "C" fn integer_receiver_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut IntegerReceiverWrapper);
    }
}

unsafe extern "C" fn byte_sender_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut ByteSenderWrapper);
    }
}

unsafe extern "C" fn byte_receiver_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut ByteReceiverWrapper);
    }
}

// Mark functions (none needed)
unsafe extern "C" fn float_sender_mark(_ptr: *mut c_void) {}
unsafe extern "C" fn float_receiver_mark(_ptr: *mut c_void) {}
unsafe extern "C" fn integer_sender_mark(_ptr: *mut c_void) {}
unsafe extern "C" fn integer_receiver_mark(_ptr: *mut c_void) {}
unsafe extern "C" fn byte_sender_mark(_ptr: *mut c_void) {}
unsafe extern "C" fn byte_receiver_mark(_ptr: *mut c_void) {}

// Helper functions to get wrappers from Ruby objects
// Using the correct rb-sys 0.9+ API: RTYPEDDATA_GET_DATA
unsafe fn get_float_sender_wrapper(obj: VALUE) -> *mut FloatSenderWrapper {
    RTYPEDDATA_GET_DATA(obj) as *mut FloatSenderWrapper
}

unsafe fn get_float_receiver_wrapper(obj: VALUE) -> *mut FloatReceiverWrapper {
    RTYPEDDATA_GET_DATA(obj) as *mut FloatReceiverWrapper
}

unsafe fn get_integer_sender_wrapper(obj: VALUE) -> *mut IntegerSenderWrapper {
    RTYPEDDATA_GET_DATA(obj) as *mut IntegerSenderWrapper
}

unsafe fn get_integer_receiver_wrapper(obj: VALUE) -> *mut IntegerReceiverWrapper {
    RTYPEDDATA_GET_DATA(obj) as *mut IntegerReceiverWrapper
}

unsafe fn get_byte_sender_wrapper(obj: VALUE) -> *mut ByteSenderWrapper {
    RTYPEDDATA_GET_DATA(obj) as *mut ByteSenderWrapper
}

unsafe fn get_byte_receiver_wrapper(obj: VALUE) -> *mut ByteReceiverWrapper {
    RTYPEDDATA_GET_DATA(obj) as *mut ByteReceiverWrapper
}

// Float Sender implementations
unsafe extern "C" fn float_sender_new(_klass: VALUE, capacity_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    
    match Sender::new(capacity) {
        Ok(sender) => {
            let wrapper = Box::new(FloatSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                FLOAT_SENDER_CLASS,
                wrapper_ptr,
                Some(float_sender_mark),
                Some(float_sender_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to create float sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn float_sender_open(_klass: VALUE, capacity_val: VALUE, memfd_val: VALUE, empty_signal_val: VALUE, full_signal_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    let memfd_num = rb_num2int(memfd_val);
    let empty_signal_num = rb_num2int(empty_signal_val);
    let full_signal_num = rb_num2int(full_signal_val);

    let memfd = File::from_raw_fd(memfd_num as i32);
    let empty_signal = File::from_raw_fd(empty_signal_num as i32);
    let full_signal = File::from_raw_fd(full_signal_num as i32);

    match Sender::open(capacity, memfd, empty_signal, full_signal) {
        Ok(sender) => {
            let wrapper = Box::new(FloatSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                FLOAT_SENDER_CLASS,
                wrapper_ptr,
                Some(float_sender_mark),
                Some(float_sender_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to open float sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn float_sender_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_float_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float sender object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let sender_ref = (*wrapper).sender.as_ref().unwrap();
    
    let memfd_fd = sender_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = sender_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = sender_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, rb_int2big(memfd_fd as isize));
    rb_ary_push(array, rb_int2big(empty_fd as isize));
    rb_ary_push(array, rb_int2big(full_fd as isize));
    array
}

unsafe extern "C" fn float_sender_send_data(_self: VALUE, ruby_array: VALUE) -> VALUE {
    let wrapper = get_float_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float sender object\0".as_ptr() as *const i8);
        unreachable!()
    }
    if RB_TYPE(ruby_array) != RUBY_T_ARRAY {
        rb_raise(rb_eTypeError, b"Expected array\0".as_ptr() as *const i8);
        unreachable!()
    }

    let len = RARRAY_LEN(ruby_array) as usize;
    let mut data = Vec::with_capacity(len);
    
    for i in 0..len {
        let elem = rb_ary_entry(ruby_array, i as c_long);
        let value = rb_float_value(elem);
        data.push(value);
    }

    let sender_ref = (*wrapper).sender.as_mut().unwrap();
    let mut data_index = 0;
    
    match sender_ref.send_raw(|ptr, count| {
        let items_to_write = std::cmp::min(count, data.len() - data_index);
        for i in 0..items_to_write {
            *ptr.add(i) = data[data_index + i];
        }
        data_index += items_to_write;
        items_to_write
    }) {
        Ok(status) => {
            let result = rb_hash_new();
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), rb_uint2big(status.remaining));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue as VALUE } else { Qfalse as VALUE });
            result
        }
        Err(e) => {
            let error_msg = format!("Send failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

// Float Receiver implementations
unsafe extern "C" fn float_receiver_new(_klass: VALUE, capacity_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    
    match Receiver::new(capacity) {
        Ok(receiver) => {
            let wrapper = Box::new(FloatReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                FLOAT_RECEIVER_CLASS,
                wrapper_ptr,
                Some(float_receiver_mark),
                Some(float_receiver_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to create float receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn float_receiver_open(_klass: VALUE, capacity_val: VALUE, memfd_val: VALUE, empty_signal_val: VALUE, full_signal_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    let memfd_num = rb_num2int(memfd_val);
    let empty_signal_num = rb_num2int(empty_signal_val);
    let full_signal_num = rb_num2int(full_signal_val);

    let memfd = File::from_raw_fd(memfd_num as i32);
    let empty_signal = File::from_raw_fd(empty_signal_num as i32);
    let full_signal = File::from_raw_fd(full_signal_num as i32);

    match Receiver::open(capacity, memfd, empty_signal, full_signal) {
        Ok(receiver) => {
            let wrapper = Box::new(FloatReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                FLOAT_RECEIVER_CLASS,
                wrapper_ptr,
                Some(float_receiver_mark),
                Some(float_receiver_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to open float receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn float_receiver_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_float_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float receiver object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let receiver_ref = (*wrapper).receiver.as_ref().unwrap();
    
    let memfd_fd = receiver_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = receiver_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = receiver_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, rb_int2big(memfd_fd as isize));
    rb_ary_push(array, rb_int2big(empty_fd as isize));
    rb_ary_push(array, rb_int2big(full_fd as isize));
    array
}

unsafe extern "C" fn float_receiver_receive_data(_self: VALUE) -> VALUE {
    let wrapper = get_float_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float receiver object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let receiver_ref = (*wrapper).receiver.as_mut().unwrap();
    let mut received_data = Vec::new();
    
    match receiver_ref.receive_raw(|ptr, count| {
        for i in 0..count {
            received_data.push(*ptr.add(i));
        }
        count
    }) {
        Ok(status) => {
            let result = rb_hash_new();
            
            let ruby_array = rb_ary_new_capa(received_data.len() as c_long);
            for value in received_data {
                rb_ary_push(ruby_array, rb_float_new(value));
            }
            
            rb_hash_aset(result, rb_str_new_cstr(b"data\0".as_ptr() as *const i8), ruby_array);
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), rb_uint2big(status.remaining));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue as VALUE } else { Qfalse as VALUE });
            result
        }
        Err(e) => {
            let error_msg = format!("Receive failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

// Integer Sender implementations
unsafe extern "C" fn integer_sender_new(_klass: VALUE, capacity_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    
    match Sender::new(capacity) {
        Ok(sender) => {
            let wrapper = Box::new(IntegerSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                INTEGER_SENDER_CLASS,
                wrapper_ptr,
                Some(integer_sender_mark),
                Some(integer_sender_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to create integer sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn integer_sender_open(_klass: VALUE, capacity_val: VALUE, memfd_val: VALUE, empty_signal_val: VALUE, full_signal_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    let memfd_num = rb_num2int(memfd_val);
    let empty_signal_num = rb_num2int(empty_signal_val);
    let full_signal_num = rb_num2int(full_signal_val);

    let memfd = File::from_raw_fd(memfd_num as i32);
    let empty_signal = File::from_raw_fd(empty_signal_num as i32);
    let full_signal = File::from_raw_fd(full_signal_num as i32);

    match Sender::open(capacity, memfd, empty_signal, full_signal) {
        Ok(sender) => {
            let wrapper = Box::new(IntegerSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                INTEGER_SENDER_CLASS,
                wrapper_ptr,
                Some(integer_sender_mark),
                Some(integer_sender_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to open integer sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn integer_sender_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_integer_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer sender object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let sender_ref = (*wrapper).sender.as_ref().unwrap();
    
    let memfd_fd = sender_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = sender_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = sender_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, rb_int2big(memfd_fd as isize));
    rb_ary_push(array, rb_int2big(empty_fd as isize));
    rb_ary_push(array, rb_int2big(full_fd as isize));
    array
}

unsafe extern "C" fn integer_sender_send_data(_self: VALUE, ruby_array: VALUE) -> VALUE {
    let wrapper = get_integer_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer sender object\0".as_ptr() as *const i8);
        unreachable!()
    }
    if RB_TYPE(ruby_array) != RUBY_T_ARRAY {
        rb_raise(rb_eTypeError, b"Expected array\0".as_ptr() as *const i8);
        unreachable!()
    }

    let len = RARRAY_LEN(ruby_array) as usize;
    let mut data = Vec::with_capacity(len);
    
    for i in 0..len {
        let elem = rb_ary_entry(ruby_array, i as c_long);
        let value = rb_num2ll(elem);
        data.push(value);
    }

    let sender_ref = (*wrapper).sender.as_mut().unwrap();
    let mut data_index = 0;
    
    match sender_ref.send_raw(|ptr, count| {
        let items_to_write = std::cmp::min(count, data.len() - data_index);
        for i in 0..items_to_write {
            *ptr.add(i) = data[data_index + i];
        }
        data_index += items_to_write;
        items_to_write
    }) {
        Ok(status) => {
            let result = rb_hash_new();
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), rb_uint2big(status.remaining));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue as VALUE } else { Qfalse as VALUE });
            result
        }
        Err(e) => {
            let error_msg = format!("Send failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

// Integer Receiver implementations
unsafe extern "C" fn integer_receiver_new(_klass: VALUE, capacity_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    
    match Receiver::new(capacity) {
        Ok(receiver) => {
            let wrapper = Box::new(IntegerReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                INTEGER_RECEIVER_CLASS,
                wrapper_ptr,
                Some(integer_receiver_mark),
                Some(integer_receiver_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to create integer receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn integer_receiver_open(_klass: VALUE, capacity_val: VALUE, memfd_val: VALUE, empty_signal_val: VALUE, full_signal_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    let memfd_num = rb_num2int(memfd_val);
    let empty_signal_num = rb_num2int(empty_signal_val);
    let full_signal_num = rb_num2int(full_signal_val);

    let memfd = File::from_raw_fd(memfd_num as i32);
    let empty_signal = File::from_raw_fd(empty_signal_num as i32);
    let full_signal = File::from_raw_fd(full_signal_num as i32);

    match Receiver::open(capacity, memfd, empty_signal, full_signal) {
        Ok(receiver) => {
            let wrapper = Box::new(IntegerReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                INTEGER_RECEIVER_CLASS,
                wrapper_ptr,
                Some(integer_receiver_mark),
                Some(integer_receiver_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to open integer receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn integer_receiver_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_integer_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer receiver object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let receiver_ref = (*wrapper).receiver.as_ref().unwrap();
    
    let memfd_fd = receiver_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = receiver_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = receiver_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, rb_int2big(memfd_fd as isize));
    rb_ary_push(array, rb_int2big(empty_fd as isize));
    rb_ary_push(array, rb_int2big(full_fd as isize));
    array
}

unsafe extern "C" fn integer_receiver_receive_data(_self: VALUE) -> VALUE {
    let wrapper = get_integer_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer receiver object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let receiver_ref = (*wrapper).receiver.as_mut().unwrap();
    let mut received_data = Vec::new();
    
    match receiver_ref.receive_raw(|ptr, count| {
        for i in 0..count {
            received_data.push(*ptr.add(i));
        }
        count
    }) {
        Ok(status) => {
            let result = rb_hash_new();
            
            let ruby_array = rb_ary_new_capa(received_data.len() as c_long);
            for value in received_data {
                rb_ary_push(ruby_array, rb_int2big(value as isize));
            }
            
            rb_hash_aset(result, rb_str_new_cstr(b"data\0".as_ptr() as *const i8), ruby_array);
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), rb_uint2big(status.remaining));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue as VALUE } else { Qfalse as VALUE });
            result
        }
        Err(e) => {
            let error_msg = format!("Receive failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

// Byte Sender implementations
unsafe extern "C" fn byte_sender_new(_klass: VALUE, capacity_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    
    match Sender::new(capacity) {
        Ok(sender) => {
            let wrapper = Box::new(ByteSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                BYTE_SENDER_CLASS,
                wrapper_ptr,
                Some(byte_sender_mark),
                Some(byte_sender_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to create byte sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn byte_sender_open(_klass: VALUE, capacity_val: VALUE, memfd_val: VALUE, empty_signal_val: VALUE, full_signal_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    let memfd_num = rb_num2int(memfd_val);
    let empty_signal_num = rb_num2int(empty_signal_val);
    let full_signal_num = rb_num2int(full_signal_val);

    let memfd = File::from_raw_fd(memfd_num as i32);
    let empty_signal = File::from_raw_fd(empty_signal_num as i32);
    let full_signal = File::from_raw_fd(full_signal_num as i32);

    match Sender::open(capacity, memfd, empty_signal, full_signal) {
        Ok(sender) => {
            let wrapper = Box::new(ByteSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                BYTE_SENDER_CLASS,
                wrapper_ptr,
                Some(byte_sender_mark),
                Some(byte_sender_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to open byte sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn byte_sender_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_byte_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte sender object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let sender_ref = (*wrapper).sender.as_ref().unwrap();
    
    let memfd_fd = sender_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = sender_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = sender_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, rb_int2big(memfd_fd as isize));
    rb_ary_push(array, rb_int2big(empty_fd as isize));
    rb_ary_push(array, rb_int2big(full_fd as isize));
    array
}

unsafe extern "C" fn byte_sender_send_data(_self: VALUE, ruby_data: VALUE) -> VALUE {
    let wrapper = get_byte_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte sender object\0".as_ptr() as *const i8);
        unreachable!()
    }
    let data_bytes = if RB_TYPE(ruby_data) == RUBY_T_STRING {
        // Handle string input
        let str_ptr = RSTRING_PTR(ruby_data);
        let str_len = RSTRING_LEN(ruby_data) as usize;
        let slice = std::slice::from_raw_parts(str_ptr as *const u8, str_len);
        slice.to_vec()
    } else if RB_TYPE(ruby_data) == RUBY_T_ARRAY {
        // Handle array of integers
        let len = RARRAY_LEN(ruby_data) as usize;
        let mut bytes = Vec::with_capacity(len);
        for i in 0..len {
            let elem = rb_ary_entry(ruby_data, i as c_long);
            let value = rb_num2int(elem) as u8;
            bytes.push(value);
        }
        bytes
    } else {
        rb_raise(rb_eTypeError, b"Expected string or array of integers\0".as_ptr() as *const i8);
        unreachable!()
    };

    let sender_ref = (*wrapper).sender.as_mut().unwrap();
    let mut data_index = 0;
    
    match sender_ref.send_raw(|ptr, count| {
        let items_to_write = std::cmp::min(count, data_bytes.len() - data_index);
        for i in 0..items_to_write {
            *ptr.add(i) = data_bytes[data_index + i];
        }
        data_index += items_to_write;
        items_to_write
    }) {
        Ok(status) => {
            let result = rb_hash_new();
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), rb_uint2big(status.remaining));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue as VALUE } else { Qfalse as VALUE });
            result
        }
        Err(e) => {
            let error_msg = format!("Send failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

// Byte Receiver implementations
unsafe extern "C" fn byte_receiver_new(_klass: VALUE, capacity_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    
    match Receiver::new(capacity) {
        Ok(receiver) => {
            let wrapper = Box::new(ByteReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                BYTE_RECEIVER_CLASS,
                wrapper_ptr,
                Some(byte_receiver_mark),
                Some(byte_receiver_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to create byte receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn byte_receiver_open(_klass: VALUE, capacity_val: VALUE, memfd_val: VALUE, empty_signal_val: VALUE, full_signal_val: VALUE) -> VALUE {
    let capacity = rb_num2ulong(capacity_val) as usize;
    let memfd_num = rb_num2int(memfd_val);
    let empty_signal_num = rb_num2int(empty_signal_val);
    let full_signal_num = rb_num2int(full_signal_val);

    let memfd = File::from_raw_fd(memfd_num as i32);
    let empty_signal = File::from_raw_fd(empty_signal_num as i32);
    let full_signal = File::from_raw_fd(full_signal_num as i32);

    match Receiver::open(capacity, memfd, empty_signal, full_signal) {
        Ok(receiver) => {
            let wrapper = Box::new(ByteReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut c_void;
            rb_data_object_wrap(
                BYTE_RECEIVER_CLASS,
                wrapper_ptr,
                Some(byte_receiver_mark),
                Some(byte_receiver_free),
            )
        }
        Err(e) => {
            let error_msg = format!("Failed to open byte receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

unsafe extern "C" fn byte_receiver_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_byte_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte receiver object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let receiver_ref = (*wrapper).receiver.as_ref().unwrap();
    
    let memfd_fd = receiver_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = receiver_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = receiver_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, rb_int2big(memfd_fd as isize));
    rb_ary_push(array, rb_int2big(empty_fd as isize));
    rb_ary_push(array, rb_int2big(full_fd as isize));
    array
}

unsafe extern "C" fn byte_receiver_receive_data(_self: VALUE) -> VALUE {
    let wrapper = get_byte_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte receiver object\0".as_ptr() as *const i8);
        unreachable!()
    }

    let receiver_ref = (*wrapper).receiver.as_mut().unwrap();
    let mut received_data = Vec::new();
    
    match receiver_ref.receive_raw(|ptr, count| {
        for i in 0..count {
            received_data.push(*ptr.add(i));
        }
        count
    }) {
        Ok(status) => {
            let result = rb_hash_new();
            
            // Return bytes as a Ruby string
            let ruby_string = rb_str_new(received_data.as_ptr() as *const i8, received_data.len() as c_long);
            
            rb_hash_aset(result, rb_str_new_cstr(b"data\0".as_ptr() as *const i8), ruby_string);
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), rb_uint2big(status.remaining));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue as VALUE } else { Qfalse as VALUE });
            result
        }
        Err(e) => {
            let error_msg = format!("Receive failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            unreachable!()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Init_shmem_ipc() {
    // Define module
    SHMEM_IPC_MODULE = rb_define_module(b"ShmemIpc\0".as_ptr() as *const i8);
    
    // Define FloatSender class
    FLOAT_SENDER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"FloatSender\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(FLOAT_SENDER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_new)), 1);
    rb_define_singleton_method(FLOAT_SENDER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_open)), 4);
    rb_define_method(FLOAT_SENDER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_get_fds)), 0);
    rb_define_method(FLOAT_SENDER_CLASS, b"send_data\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_send_data as *const ())), 1);
    
    // Define FloatReceiver class
    FLOAT_RECEIVER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"FloatReceiver\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(FLOAT_RECEIVER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_new)), 1);
    rb_define_singleton_method(FLOAT_RECEIVER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_open)), 4);
    rb_define_method(FLOAT_RECEIVER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_get_fds)), 0);
    rb_define_method(FLOAT_RECEIVER_CLASS, b"receive_data\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_receive_data)), 0);
    
    // Define IntegerSender class
    INTEGER_SENDER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"IntegerSender\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(INTEGER_SENDER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_new)), 1);
    rb_define_singleton_method(INTEGER_SENDER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_open)), 4);
    rb_define_method(INTEGER_SENDER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_get_fds)), 0);
    rb_define_method(INTEGER_SENDER_CLASS, b"send_data\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_send_data)), 1);
    
    // Define IntegerReceiver class
    INTEGER_RECEIVER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"IntegerReceiver\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(INTEGER_RECEIVER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_new)), 1);
    rb_define_singleton_method(INTEGER_RECEIVER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_open)), 4);
    rb_define_method(INTEGER_RECEIVER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_get_fds)), 0);
    rb_define_method(INTEGER_RECEIVER_CLASS, b"receive_data\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_receive_data)), 0);
    
    // Define ByteSender class
    BYTE_SENDER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"ByteSender\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(BYTE_SENDER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_new)), 1);
    rb_define_singleton_method(BYTE_SENDER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_open)), 4);
    rb_define_method(BYTE_SENDER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_get_fds)), 0);
    rb_define_method(BYTE_SENDER_CLASS, b"send_data\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_send_data)), 1);
    
    // Define ByteReceiver class
    BYTE_RECEIVER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"ByteReceiver\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(BYTE_RECEIVER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_new)), 1);
    rb_define_singleton_method(BYTE_RECEIVER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_open)), 4);
    rb_define_method(BYTE_RECEIVER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_get_fds)), 0);
    rb_define_method(BYTE_RECEIVER_CLASS, b"receive_data\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_receive_data)), 0);
}