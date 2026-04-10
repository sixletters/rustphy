;; (module
;;     ;; Only one memory declaration allowed
;;     (func $log (import "imports" "log") (param i32))
;;     (memory 1 10) ;; initial 1 page; max 10 pages

;;     ;; Define a type for binary operations (i32, i32) -> i32
;;     ;; This is required for call_indirect
;;     (type $binary_op (func (param i32 i32) (result i32)))

;;     ;; global heap_ptr starts at byte 1000
;;     (global $heap_ptr (mut i32) (i32.const 1000))

;;     (func $alloc (param $size i32) (result i32)
;;         (local $ptr i32)

;;         ;; save current heap_ptr
;;         global.get $heap_ptr
;;         local.set $ptr

;;         ;; bump heap pointer forward
;;         global.get $heap_ptr
;;         local.get $size
;;         i32.add
;;         global.set $heap_ptr

;;         local.get $ptr
;;     )

;;     (func $alloc_string (param $length i32) (result i32)
;;         (local $ptr i32)
;;         (local $total_size i32)

;;         ;; calculate total size -> 8 bytes header + length, 4 bytes for type tag and then next 4 bytes for length
;;         i32.const 8
;;         local.get $length
;;         i32.add
;;         local.set $total_size

;;         ;; allocate memory
;;         local.get $total_size
;;         call $alloc
;;         local.set $ptr

;;         ;; write type tag
;;         local.get $ptr
;;         i32.const 1
;;         i32.store

;;         ;; write length
;;         local.get $ptr
;;         local.get $length
;;         i32.store offset=4

;;         ;; return pointer
;;         local.get $ptr
;;     )

;;     ;; byte addressable
;;     ;; 1 page is usually 64 KB

;;     (func $storeStuff
;;         ;; store takes expeces [address, value] ->top of stack
;;         i32.const 100
;;         i32.const 42
;;         i32.store

;;         ;; i32.store8 - Store 1 byte
;;         i32.const 200     ;; Address
;;         i32.const 65      ;; Value (ASCII 'A')
;;         i32.store8        ;; memory[200] = 65

;;         ;; i32.store16 - Store 2 bytes
;;         i32.const 300
;;         i32.const 1000
;;         i32.store16       ;; memory[300..301] = 1000 (little-endian)


;;         ;; Store at base + offset
;;         i32.const 100     ;; Base address
;;         i32.const 42      ;; Value
;;         i32.store offset=4  ;; Store at memory[104..107]


;;         ;; Reading data from memory
;;         i32.const 100
;;         i32.load   ;; Stack: [value from memory[100..103]]

;;         i32.const 100
;;         i32.load offset=4  ;; Load from memory[104..107]

;;         drop
;;         drop
;;     )

;;     (table 2 funcref)

;;     ;; Fixed: added i32 return type
;;     (func $add (param i32 i32) (result i32)
;;         local.get 0
;;         local.get 1
;;         i32.add
;;     )

;;     (func $mul (param i32 i32) (result i32)
;;         local.get 0
;;         local.get 1
;;         i32.mul
;;     )

;;     (elem (i32.const 0) $add $mul)

;;     ;; Fixed: use type reference instead of inline type
;;     (func $dispatch (param $op i32) (param $a i32) (param $b i32) (result i32)
;;         local.get $a
;;         local.get $b
;;         local.get $op
;;         ;; call function at table[$op] with type (i32, i32) -> i32
;;         ;; Reference the type by name or index (0)
;;         call_indirect (type $binary_op)
;;     )

;;     ;; Key difference between memory and tables, memory = array of bytes
;;     ;; tables = array of references (store functions pointers or object)
;;     ;; tables solve a critical problem, indirect function calls (function pointers)
;;     ;; in wasm, you cannot directly store function references in memory, you must use tables
;;     ;; use cases include -> function pointers, vtables, callbacks, dynamic dispatch
;;     ;; indirect calls - call different functions based on runtime values
;;     ;; jump tables - switch statements
;;     ;; method calls

;;     ( ;; Table for function pointers
;;         (table $closure 10 funcref)

;;         ;; Memory for closure data
;;     )

;;     (func $test (export "test") 
;;       i32.const 0
;;       i32.const 5
;;       i32.const 8
;;       call $dispatch
;;       call $log
;;     )

;;    (func (export "_start")
;;       call $storeStuff
;;       i32.const 0
;;       i32.const 5
;;       i32.const 8
;;       call $dispatch
;;       call $log
;;    )
;; )
(module
    (import "imports" "log" (func $log (param i32)))
    (memory 1)
    (export "memory" (memory 0))

    (type $ret_i32 (func (param i32 i32) (result i32)))

    (type $closure_type (func (param i32 i32) (result i32)))

    ;; Create a table
    (table $closures 5 funcref)

    ;; Global heap ptr
    (global $heap_ptr (mut i32) (i32.const 1024))

    ;; Initialize table
    (elem (i32.const 0) $lambda_0)



    ;; Create an env
    ;; env are of the form [parent_ptr, var count, ptr_to_var_1, ...]
    (func $create_env (param $parent_ptr i32) (param $var_count i32) (result i32)
        (local $env_ptr i32)
        (local $size i32)

        ;; Have to allocate 8 + var_count
        i32.const 8
        local.get $var_count
        i32.const 4
        i32.mul
        i32.add

        ;; store expects -> [ptr, value] on the stack
        call $alloc
        local.set $env_ptr

        ;; store the parent pointer
        local.get $env_ptr
        local.get $parent_ptr
        i32.store

        ;; store var count
        local.get $env_ptr
        local.get $var_count
        i32.store offset=4

        local.get $env_ptr
    )

    ;; Example goophy code
;; fn double(x) {
;;     return x + x
;; }

;; let f = double
;; let result = f(5)
;; print(result)  // Should print 10

    (func $env_get (param $env_ptr i32) (param $index i32) (result i32)
        local.get $env_ptr
        i32.const 8
        local.get $index
        i32.const 4
        i32.mul
        i32.add
        i32.add
        i32.load
    )

    (func $env_set (param $env_ptr i32) (param $index i32) (param $value i32)
        local.get $env_ptr
        i32.const 8
        local.get $index
        i32.const 4
        i32.mul
        i32.add
        i32.add
        local.get $value
        i32.store
    )

    ;; A closure is of the form [function_idx (4 bytes), env pointer (4 bytes)]
    (func $create_closure (param $func_idx i32) (param $env_ptr i32) (result i32)
        (local $closure_ptr i32)

        ;; allocate 8 bytes on the heap
        i32.const 8
        call $alloc
        
        ;; set the closure_ptr
        local.set $closure_ptr

        ;; store func idx
        local.get $closure_ptr
        local.get $func_idx
        i32.store

        ;; store env ptr
        local.get $closure_ptr
        local.get $env_ptr
        i32.store offset=4

        local.get $closure_ptr
    )

    (func $call_closure (param $closure_ptr i32) (param $arg i32) (result i32)
        (local $func_idx i32)
        (local $env i32)

        ;; get the func index
        local.get $closure_ptr
        i32.load
        local.set $func_idx

        ;; get the env pointr
        local.get $closure_ptr
        i32.load offset=4
        local.set $env

        ;; call via table 
        local.get $env
        local.get $arg
        local.get $func_idx
        call_indirect (type $ret_i32)
    )

    ;; makes the adder function with x as a pointer
    (func $makeAdder (param $x i32) (result i32)
        (local $env i32)

        ;; Create env with 1 var
        i32.const 0;; no parent
        i32.const 1 ;; var count
        call $create_env
        local.set $env

        ;; Store 'x' in environment
        local.get $env
        i32.const 0      ;; Variable index 0
        local.get $x
        call $create_integer
        call $env_set

        ;; Create closure
        i32.const 0      ;; Function table index for $lambda_0
        local.get $env
        call $create_closure
    )

    ;; This implements something like this
    ;; x = 1
    ;; func(y) {
    ;;  return x + y;
    ;; }
    (func $lambda_0 (param $env_ptr i32) (param $y i32) (result i32)
        (local $x i32)

        ;; Load x from the env
        ;; env is stored such that it is
        ;; lets say x is at the 0th index
        local.get $env_ptr
        i32.const 0
        call $env_get
        local.set $x

        local.get $x
        call $get_data
        call $untag_integer


        local.get $y
        call $get_data
        call $untag_integer

        i32.add
        call $create_integer
    )

    (func $alloc (param $length i32) (result i32)
        (local $ptr i32)
        global.get $heap_ptr
        local.set $ptr

        local.get $length
        global.get $heap_ptr
        i32.add
        global.set $heap_ptr

        local.get $ptr
    )

    ;; integer will be tagged with 00 as the most right bits and allocated on the heap
    (func $create_integer (param $value i32) (result i32)
        (local $ptr i32)

        ;; integer is 4 bytes
        i32.const 4
        call $alloc
        local.set $ptr

        local.get $ptr
        local.get $value
        call $tag_integer
        i32.store

        local.get $ptr
    )

    ;; tag integer with 00 as the most right bits
    (func $tag_integer (param $value i32) (result i32)
        local.get $value
        i32.const 2
        i32.shl
    )

    ;; untag integer with 00 as the most right bits
    (func $untag_integer (param $value i32) (result i32)
        local.get $value
        i32.const 2
        i32.shr_s
    )

    (func $get_data (param $ptr i32) (result i32)
        local.get $ptr
        i32.load
    )

    (func $add_integer (export "add_integer") (param $x i32) (param $y i32) (result i32)
        local.get $x
        call $get_data
        call $untag_integer

        local.get $y
        call $get_data
        call $untag_integer

        i32.add
    )

    (func $main (export "main") (result i32)
        ;; let add10 = makeAdder(10)
        i32.const 10
        call $makeAdder

        ;; add10(5)
        i32.const 5
        call $create_integer
        call $call_closure

        call $get_data
        call $untag_integer
        call $log

        i32.const 1
    )
)
