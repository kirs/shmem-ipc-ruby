use rb_sys::*;
use shmem_ipc::sharedring::{Receiver, Sender};
use std::ffi::CStr;
use std::fs::File;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::ptr;

// Ruby class definitions
static mut SHMEM_IPC_MODULE: VALUE = QNIL;
static mut FLOAT_SENDER_CLASS: VALUE = QNIL;
static mut FLOAT_RECEIVER_CLASS: VALUE = QNIL;
static mut INTEGER_SENDER_CLASS: VALUE = QNIL;
static mut INTEGER_RECEIVER_CLASS: VALUE = QNIL;
static mut BYTE_SENDER_CLASS: VALUE = QNIL;
static mut BYTE_RECEIVER_CLASS: VALUE = QNIL;

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
unsafe extern "C" fn float_sender_free(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut FloatSenderWrapper);
    }
}

unsafe extern "C" fn float_receiver_free(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut FloatReceiverWrapper);
    }
}

unsafe extern "C" fn integer_sender_free(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut IntegerSenderWrapper);
    }
}

unsafe extern "C" fn integer_receiver_free(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut IntegerReceiverWrapper);
    }
}

unsafe extern "C" fn byte_sender_free(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut ByteSenderWrapper);
    }
}

unsafe extern "C" fn byte_receiver_free(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let _wrapper = Box::from_raw(ptr as *mut ByteReceiverWrapper);
    }
}

// Mark functions (none needed)
unsafe extern "C" fn float_sender_mark(_ptr: *mut std::ffi::c_void) {}
unsafe extern "C" fn float_receiver_mark(_ptr: *mut std::ffi::c_void) {}
unsafe extern "C" fn integer_sender_mark(_ptr: *mut std::ffi::c_void) {}
unsafe extern "C" fn integer_receiver_mark(_ptr: *mut std::ffi::c_void) {}
unsafe extern "C" fn byte_sender_mark(_ptr: *mut std::ffi::c_void) {}
unsafe extern "C" fn byte_receiver_mark(_ptr: *mut std::ffi::c_void) {}

// Data types for Ruby
static FLOAT_SENDER_DATA_TYPE: rb_data_type_t = rb_data_type_t {
    wrap_struct_name: b"ShmemIpc::FloatSender\0".as_ptr() as *const i8,
    function: rb_data_type_struct {
        dmark: Some(float_sender_mark),
        dfree: Some(float_sender_free),
        dsize: None,
        dcompact: None,
        reserved: [ptr::null_mut(); 1],
    },
    parent: ptr::null(),
    data: ptr::null_mut(),
    flags: RUBY_TYPED_FREE_IMMEDIATELY,
};

static FLOAT_RECEIVER_DATA_TYPE: rb_data_type_t = rb_data_type_t {
    wrap_struct_name: b"ShmemIpc::FloatReceiver\0".as_ptr() as *const i8,
    function: rb_data_type_struct {
        dmark: Some(float_receiver_mark),
        dfree: Some(float_receiver_free),
        dsize: None,
        dcompact: None,
        reserved: [ptr::null_mut(); 1],
    },
    parent: ptr::null(),
    data: ptr::null_mut(),
    flags: RUBY_TYPED_FREE_IMMEDIATELY,
};

static INTEGER_SENDER_DATA_TYPE: rb_data_type_t = rb_data_type_t {
    wrap_struct_name: b"ShmemIpc::IntegerSender\0".as_ptr() as *const i8,
    function: rb_data_type_struct {
        dmark: Some(integer_sender_mark),
        dfree: Some(integer_sender_free),
        dsize: None,
        dcompact: None,
        reserved: [ptr::null_mut(); 1],
    },
    parent: ptr::null(),
    data: ptr::null_mut(),
    flags: RUBY_TYPED_FREE_IMMEDIATELY,
};

static INTEGER_RECEIVER_DATA_TYPE: rb_data_type_t = rb_data_type_t {
    wrap_struct_name: b"ShmemIpc::IntegerReceiver\0".as_ptr() as *const i8,
    function: rb_data_type_struct {
        dmark: Some(integer_receiver_mark),
        dfree: Some(integer_receiver_free),
        dsize: None,
        dcompact: None,
        reserved: [ptr::null_mut(); 1],
    },
    parent: ptr::null(),
    data: ptr::null_mut(),
    flags: RUBY_TYPED_FREE_IMMEDIATELY,
};

static BYTE_SENDER_DATA_TYPE: rb_data_type_t = rb_data_type_t {
    wrap_struct_name: b"ShmemIpc::ByteSender\0".as_ptr() as *const i8,
    function: rb_data_type_struct {
        dmark: Some(byte_sender_mark),
        dfree: Some(byte_sender_free),
        dsize: None,
        dcompact: None,
        reserved: [ptr::null_mut(); 1],
    },
    parent: ptr::null(),
    data: ptr::null_mut(),
    flags: RUBY_TYPED_FREE_IMMEDIATELY,
};

static BYTE_RECEIVER_DATA_TYPE: rb_data_type_t = rb_data_type_t {
    wrap_struct_name: b"ShmemIpc::ByteReceiver\0".as_ptr() as *const i8,
    function: rb_data_type_struct {
        dmark: Some(byte_receiver_mark),
        dfree: Some(byte_receiver_free),
        dsize: None,
        dcompact: None,
        reserved: [ptr::null_mut(); 1],
    },
    parent: ptr::null(),
    data: ptr::null_mut(),
    flags: RUBY_TYPED_FREE_IMMEDIATELY,
};

// Helper functions to get wrappers
unsafe fn get_float_sender_wrapper(obj: VALUE) -> *mut FloatSenderWrapper {
    let mut ptr: *mut std::ffi::c_void = ptr::null_mut();
    rb_data_object_get(obj, &mut ptr, &FLOAT_SENDER_DATA_TYPE);
    ptr as *mut FloatSenderWrapper
}

unsafe fn get_float_receiver_wrapper(obj: VALUE) -> *mut FloatReceiverWrapper {
    let mut ptr: *mut std::ffi::c_void = ptr::null_mut();
    rb_data_object_get(obj, &mut ptr, &FLOAT_RECEIVER_DATA_TYPE);
    ptr as *mut FloatReceiverWrapper
}

unsafe fn get_integer_sender_wrapper(obj: VALUE) -> *mut IntegerSenderWrapper {
    let mut ptr: *mut std::ffi::c_void = ptr::null_mut();
    rb_data_object_get(obj, &mut ptr, &INTEGER_SENDER_DATA_TYPE);
    ptr as *mut IntegerSenderWrapper
}

unsafe fn get_integer_receiver_wrapper(obj: VALUE) -> *mut IntegerReceiverWrapper {
    let mut ptr: *mut std::ffi::c_void = ptr::null_mut();
    rb_data_object_get(obj, &mut ptr, &INTEGER_RECEIVER_DATA_TYPE);
    ptr as *mut IntegerReceiverWrapper
}

unsafe fn get_byte_sender_wrapper(obj: VALUE) -> *mut ByteSenderWrapper {
    let mut ptr: *mut std::ffi::c_void = ptr::null_mut();
    rb_data_object_get(obj, &mut ptr, &BYTE_SENDER_DATA_TYPE);
    ptr as *mut ByteSenderWrapper
}

unsafe fn get_byte_receiver_wrapper(obj: VALUE) -> *mut ByteReceiverWrapper {
    let mut ptr: *mut std::ffi::c_void = ptr::null_mut();
    rb_data_object_get(obj, &mut ptr, &BYTE_RECEIVER_DATA_TYPE);
    ptr as *mut ByteReceiverWrapper
}

// Float Sender implementations
unsafe extern "C" fn float_sender_new(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv) as usize;
    
    match Sender::new(capacity) {
        Ok(sender) => {
            let wrapper = Box::new(FloatSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(FLOAT_SENDER_CLASS, wrapper_ptr, &FLOAT_SENDER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to create float sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn float_sender_open(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 4 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 4)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv.offset(0)) as usize;
    let memfd_num = NUM2INT(*argv.offset(1));
    let empty_signal_num = NUM2INT(*argv.offset(2));
    let full_signal_num = NUM2INT(*argv.offset(3));

    let memfd = File::from_raw_fd(memfd_num);
    let empty_signal = File::from_raw_fd(empty_signal_num);
    let full_signal = File::from_raw_fd(full_signal_num);

    match Sender::open(capacity, memfd, empty_signal, full_signal) {
        Ok(sender) => {
            let wrapper = Box::new(FloatSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(FLOAT_SENDER_CLASS, wrapper_ptr, &FLOAT_SENDER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to open float sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn float_sender_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_float_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float sender object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let sender_ref = (*wrapper).sender.as_ref().unwrap();
    
    let memfd_fd = sender_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = sender_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = sender_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, INT2NUM(memfd_fd));
    rb_ary_push(array, INT2NUM(empty_fd));
    rb_ary_push(array, INT2NUM(full_fd));
    array
}

unsafe extern "C" fn float_sender_send_data(argc: c_int, argv: *const VALUE, _self: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let wrapper = get_float_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float sender object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let ruby_array = *argv;
    if rb_type(ruby_array) != RUBY_T_ARRAY {
        rb_raise(rb_eTypeError, b"Expected array\0".as_ptr() as *const i8);
        return QNIL;
    }

    let len = RARRAY_LEN(ruby_array) as usize;
    let mut data = Vec::with_capacity(len);
    
    for i in 0..len {
        let elem = rb_ary_entry(ruby_array, i as c_long);
        let value = NUM2DBL(elem);
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
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), ULONG2NUM(status.remaining as c_ulong));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue } else { Qfalse });
            result
        }
        Err(e) => {
            let error_msg = format!("Send failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

// Float Receiver implementations
unsafe extern "C" fn float_receiver_new(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv) as usize;
    
    match Receiver::new(capacity) {
        Ok(receiver) => {
            let wrapper = Box::new(FloatReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(FLOAT_RECEIVER_CLASS, wrapper_ptr, &FLOAT_RECEIVER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to create float receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn float_receiver_open(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 4 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 4)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv.offset(0)) as usize;
    let memfd_num = NUM2INT(*argv.offset(1));
    let empty_signal_num = NUM2INT(*argv.offset(2));
    let full_signal_num = NUM2INT(*argv.offset(3));

    let memfd = File::from_raw_fd(memfd_num);
    let empty_signal = File::from_raw_fd(empty_signal_num);
    let full_signal = File::from_raw_fd(full_signal_num);

    match Receiver::open(capacity, memfd, empty_signal, full_signal) {
        Ok(receiver) => {
            let wrapper = Box::new(FloatReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(FLOAT_RECEIVER_CLASS, wrapper_ptr, &FLOAT_RECEIVER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to open float receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn float_receiver_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_float_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float receiver object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let receiver_ref = (*wrapper).receiver.as_ref().unwrap();
    
    let memfd_fd = receiver_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = receiver_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = receiver_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, INT2NUM(memfd_fd));
    rb_ary_push(array, INT2NUM(empty_fd));
    rb_ary_push(array, INT2NUM(full_fd));
    array
}

unsafe extern "C" fn float_receiver_receive_data(_self: VALUE) -> VALUE {
    let wrapper = get_float_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid float receiver object\0".as_ptr() as *const i8);
        return QNIL;
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
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), ULONG2NUM(status.remaining as c_ulong));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue } else { Qfalse });
            result
        }
        Err(e) => {
            let error_msg = format!("Receive failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

// Integer Sender implementations
unsafe extern "C" fn integer_sender_new(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv) as usize;
    
    match Sender::new(capacity) {
        Ok(sender) => {
            let wrapper = Box::new(IntegerSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(INTEGER_SENDER_CLASS, wrapper_ptr, &INTEGER_SENDER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to create integer sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn integer_sender_open(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 4 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 4)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv.offset(0)) as usize;
    let memfd_num = NUM2INT(*argv.offset(1));
    let empty_signal_num = NUM2INT(*argv.offset(2));
    let full_signal_num = NUM2INT(*argv.offset(3));

    let memfd = File::from_raw_fd(memfd_num);
    let empty_signal = File::from_raw_fd(empty_signal_num);
    let full_signal = File::from_raw_fd(full_signal_num);

    match Sender::open(capacity, memfd, empty_signal, full_signal) {
        Ok(sender) => {
            let wrapper = Box::new(IntegerSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(INTEGER_SENDER_CLASS, wrapper_ptr, &INTEGER_SENDER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to open integer sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn integer_sender_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_integer_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer sender object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let sender_ref = (*wrapper).sender.as_ref().unwrap();
    
    let memfd_fd = sender_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = sender_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = sender_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, INT2NUM(memfd_fd));
    rb_ary_push(array, INT2NUM(empty_fd));
    rb_ary_push(array, INT2NUM(full_fd));
    array
}

unsafe extern "C" fn integer_sender_send_data(argc: c_int, argv: *const VALUE, _self: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let wrapper = get_integer_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer sender object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let ruby_array = *argv;
    if rb_type(ruby_array) != RUBY_T_ARRAY {
        rb_raise(rb_eTypeError, b"Expected array\0".as_ptr() as *const i8);
        return QNIL;
    }

    let len = RARRAY_LEN(ruby_array) as usize;
    let mut data = Vec::with_capacity(len);
    
    for i in 0..len {
        let elem = rb_ary_entry(ruby_array, i as c_long);
        let value = NUM2LL(elem);
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
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), ULONG2NUM(status.remaining as c_ulong));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue } else { Qfalse });
            result
        }
        Err(e) => {
            let error_msg = format!("Send failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

// Integer Receiver implementations
unsafe extern "C" fn integer_receiver_new(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv) as usize;
    
    match Receiver::new(capacity) {
        Ok(receiver) => {
            let wrapper = Box::new(IntegerReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(INTEGER_RECEIVER_CLASS, wrapper_ptr, &INTEGER_RECEIVER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to create integer receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn integer_receiver_open(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 4 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 4)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv.offset(0)) as usize;
    let memfd_num = NUM2INT(*argv.offset(1));
    let empty_signal_num = NUM2INT(*argv.offset(2));
    let full_signal_num = NUM2INT(*argv.offset(3));

    let memfd = File::from_raw_fd(memfd_num);
    let empty_signal = File::from_raw_fd(empty_signal_num);
    let full_signal = File::from_raw_fd(full_signal_num);

    match Receiver::open(capacity, memfd, empty_signal, full_signal) {
        Ok(receiver) => {
            let wrapper = Box::new(IntegerReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(INTEGER_RECEIVER_CLASS, wrapper_ptr, &INTEGER_RECEIVER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to open integer receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn integer_receiver_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_integer_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer receiver object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let receiver_ref = (*wrapper).receiver.as_ref().unwrap();
    
    let memfd_fd = receiver_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = receiver_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = receiver_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, INT2NUM(memfd_fd));
    rb_ary_push(array, INT2NUM(empty_fd));
    rb_ary_push(array, INT2NUM(full_fd));
    array
}

unsafe extern "C" fn integer_receiver_receive_data(_self: VALUE) -> VALUE {
    let wrapper = get_integer_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid integer receiver object\0".as_ptr() as *const i8);
        return QNIL;
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
                rb_ary_push(ruby_array, LL2NUM(value));
            }
            
            rb_hash_aset(result, rb_str_new_cstr(b"data\0".as_ptr() as *const i8), ruby_array);
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), ULONG2NUM(status.remaining as c_ulong));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue } else { Qfalse });
            result
        }
        Err(e) => {
            let error_msg = format!("Receive failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

// Byte Sender implementations
unsafe extern "C" fn byte_sender_new(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv) as usize;
    
    match Sender::new(capacity) {
        Ok(sender) => {
            let wrapper = Box::new(ByteSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(BYTE_SENDER_CLASS, wrapper_ptr, &BYTE_SENDER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to create byte sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn byte_sender_open(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 4 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 4)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv.offset(0)) as usize;
    let memfd_num = NUM2INT(*argv.offset(1));
    let empty_signal_num = NUM2INT(*argv.offset(2));
    let full_signal_num = NUM2INT(*argv.offset(3));

    let memfd = File::from_raw_fd(memfd_num);
    let empty_signal = File::from_raw_fd(empty_signal_num);
    let full_signal = File::from_raw_fd(full_signal_num);

    match Sender::open(capacity, memfd, empty_signal, full_signal) {
        Ok(sender) => {
            let wrapper = Box::new(ByteSenderWrapper {
                sender: Some(sender),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(BYTE_SENDER_CLASS, wrapper_ptr, &BYTE_SENDER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to open byte sender: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn byte_sender_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_byte_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte sender object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let sender_ref = (*wrapper).sender.as_ref().unwrap();
    
    let memfd_fd = sender_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = sender_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = sender_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, INT2NUM(memfd_fd));
    rb_ary_push(array, INT2NUM(empty_fd));
    rb_ary_push(array, INT2NUM(full_fd));
    array
}

unsafe extern "C" fn byte_sender_send_data(argc: c_int, argv: *const VALUE, _self: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let wrapper = get_byte_sender_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte sender object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let ruby_data = *argv;
    let data_bytes = if rb_type(ruby_data) == RUBY_T_STRING {
        // Handle string input
        let str_ptr = RSTRING_PTR(ruby_data);
        let str_len = RSTRING_LEN(ruby_data) as usize;
        let slice = std::slice::from_raw_parts(str_ptr as *const u8, str_len);
        slice.to_vec()
    } else if rb_type(ruby_data) == RUBY_T_ARRAY {
        // Handle array of integers
        let len = RARRAY_LEN(ruby_data) as usize;
        let mut bytes = Vec::with_capacity(len);
        for i in 0..len {
            let elem = rb_ary_entry(ruby_data, i as c_long);
            let value = NUM2CHR(elem) as u8;
            bytes.push(value);
        }
        bytes
    } else {
        rb_raise(rb_eTypeError, b"Expected string or array of integers\0".as_ptr() as *const i8);
        return QNIL;
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
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), ULONG2NUM(status.remaining as c_ulong));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue } else { Qfalse });
            result
        }
        Err(e) => {
            let error_msg = format!("Send failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

// Byte Receiver implementations
unsafe extern "C" fn byte_receiver_new(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 1 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 1)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv) as usize;
    
    match Receiver::new(capacity) {
        Ok(receiver) => {
            let wrapper = Box::new(ByteReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(BYTE_RECEIVER_CLASS, wrapper_ptr, &BYTE_RECEIVER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to create byte receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn byte_receiver_open(argc: c_int, argv: *const VALUE, _klass: VALUE) -> VALUE {
    if argc != 4 {
        rb_raise(rb_eArgError, b"wrong number of arguments (given %d, expected 4)\0".as_ptr() as *const i8, argc);
        return QNIL;
    }

    let capacity = NUM2ULONG(*argv.offset(0)) as usize;
    let memfd_num = NUM2INT(*argv.offset(1));
    let empty_signal_num = NUM2INT(*argv.offset(2));
    let full_signal_num = NUM2INT(*argv.offset(3));

    let memfd = File::from_raw_fd(memfd_num);
    let empty_signal = File::from_raw_fd(empty_signal_num);
    let full_signal = File::from_raw_fd(full_signal_num);

    match Receiver::open(capacity, memfd, empty_signal, full_signal) {
        Ok(receiver) => {
            let wrapper = Box::new(ByteReceiverWrapper {
                receiver: Some(receiver),
            });
            let wrapper_ptr = Box::into_raw(wrapper) as *mut std::ffi::c_void;
            rb_data_typed_object_wrap(BYTE_RECEIVER_CLASS, wrapper_ptr, &BYTE_RECEIVER_DATA_TYPE)
        }
        Err(e) => {
            let error_msg = format!("Failed to open byte receiver: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

unsafe extern "C" fn byte_receiver_get_fds(_self: VALUE) -> VALUE {
    let wrapper = get_byte_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte receiver object\0".as_ptr() as *const i8);
        return QNIL;
    }

    let receiver_ref = (*wrapper).receiver.as_ref().unwrap();
    
    let memfd_fd = receiver_ref.memfd().as_file().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let empty_fd = receiver_ref.empty_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);
    let full_fd = receiver_ref.full_signal().try_clone().map(|f| f.into_raw_fd()).unwrap_or(-1);

    let array = rb_ary_new_capa(3);
    rb_ary_push(array, INT2NUM(memfd_fd));
    rb_ary_push(array, INT2NUM(empty_fd));
    rb_ary_push(array, INT2NUM(full_fd));
    array
}

unsafe extern "C" fn byte_receiver_receive_data(_self: VALUE) -> VALUE {
    let wrapper = get_byte_receiver_wrapper(_self);
    if wrapper.is_null() {
        rb_raise(rb_eRuntimeError, b"Invalid byte receiver object\0".as_ptr() as *const i8);
        return QNIL;
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
            rb_hash_aset(result, rb_str_new_cstr(b"remaining\0".as_ptr() as *const i8), ULONG2NUM(status.remaining as c_ulong));
            rb_hash_aset(result, rb_str_new_cstr(b"signal\0".as_ptr() as *const i8), if status.signal { Qtrue } else { Qfalse });
            result
        }
        Err(e) => {
            let error_msg = format!("Receive failed: {:?}\0", e);
            rb_raise(rb_eRuntimeError, error_msg.as_ptr() as *const i8);
            QNIL
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Init_shmem_ipc() {
    // Define module
    SHMEM_IPC_MODULE = rb_define_module(b"ShmemIpc\0".as_ptr() as *const i8);
    
    // Define FloatSender class
    FLOAT_SENDER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"FloatSender\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(FLOAT_SENDER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_new as *const ())), 1);
    rb_define_singleton_method(FLOAT_SENDER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_open as *const ())), 4);
    rb_define_method(FLOAT_SENDER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_get_fds as *const ())), 0);
    rb_define_method(FLOAT_SENDER_CLASS, b"send_data\0".as_ptr() as *const i8, Some(std::mem::transmute(float_sender_send_data as *const ())), 1);
    
    // Define FloatReceiver class
    FLOAT_RECEIVER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"FloatReceiver\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(FLOAT_RECEIVER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_new as *const ())), 1);
    rb_define_singleton_method(FLOAT_RECEIVER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_open as *const ())), 4);
    rb_define_method(FLOAT_RECEIVER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_get_fds as *const ())), 0);
    rb_define_method(FLOAT_RECEIVER_CLASS, b"receive_data\0".as_ptr() as *const i8, Some(std::mem::transmute(float_receiver_receive_data as *const ())), 0);
    
    // Define IntegerSender class
    INTEGER_SENDER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"IntegerSender\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(INTEGER_SENDER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_new as *const ())), 1);
    rb_define_singleton_method(INTEGER_SENDER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_open as *const ())), 4);
    rb_define_method(INTEGER_SENDER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_get_fds as *const ())), 0);
    rb_define_method(INTEGER_SENDER_CLASS, b"send_data\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_sender_send_data as *const ())), 1);
    
    // Define IntegerReceiver class
    INTEGER_RECEIVER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"IntegerReceiver\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(INTEGER_RECEIVER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_new as *const ())), 1);
    rb_define_singleton_method(INTEGER_RECEIVER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_open as *const ())), 4);
    rb_define_method(INTEGER_RECEIVER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_get_fds as *const ())), 0);
    rb_define_method(INTEGER_RECEIVER_CLASS, b"receive_data\0".as_ptr() as *const i8, Some(std::mem::transmute(integer_receiver_receive_data as *const ())), 0);
    
    // Define ByteSender class
    BYTE_SENDER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"ByteSender\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(BYTE_SENDER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_new as *const ())), 1);
    rb_define_singleton_method(BYTE_SENDER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_open as *const ())), 4);
    rb_define_method(BYTE_SENDER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_get_fds as *const ())), 0);
    rb_define_method(BYTE_SENDER_CLASS, b"send_data\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_sender_send_data as *const ())), 1);
    
    // Define ByteReceiver class
    BYTE_RECEIVER_CLASS = rb_define_class_under(SHMEM_IPC_MODULE, b"ByteReceiver\0".as_ptr() as *const i8, rb_cObject);
    rb_define_singleton_method(BYTE_RECEIVER_CLASS, b"new\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_new as *const ())), 1);
    rb_define_singleton_method(BYTE_RECEIVER_CLASS, b"open\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_open as *const ())), 4);
    rb_define_method(BYTE_RECEIVER_CLASS, b"get_fds\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_get_fds as *const ())), 0);
    rb_define_method(BYTE_RECEIVER_CLASS, b"receive_data\0".as_ptr() as *const i8, Some(std::mem::transmute(byte_receiver_receive_data as *const ())), 0);
}