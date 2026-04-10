;; ============================================
;; EXAMPLE GOOPHY CODE
;; ============================================
;; fn double(x) {
;;     return x + x
;; }
;;
;; let f = double
;; let result = f(5)
;; print(result)  // Should print 10
;;
;; ============================================
;; DESIGN IMPROVEMENTS FOR FUTURE
;; ============================================
;;
;; Current Structure:
;;   - Initialization happens in main() (not ideal)
;;   - Functions defined inline with helpers
;;   - No clear organization
;;
;; Better Structure:
;;   1. Use (start $__init) to initialize globals once at module load
;;   2. Create global closures: (global $G_double (mut i32) ...)
;;   3. Initialize in $__init, not in main
;;   4. Organize into clear sections:
;;      - Imports
;;      - Types
;;      - Memory & Globals
;;      - Function Table
;;      - Initialization ($__init + start)
;;      - Application Functions
;;      - Runtime Helpers (grouped by category)
;;
;; Example Init Pattern:
;;   (global $G_double (mut i32) (i32.const 0))
;;   (func $__init
;;       ;; Create global env
;;       i32.const 0
;;       i32.const 0
;;       call $create_env
;;       global.set $global_env
;;
;;       ;; Init closure for double
;;       i32.const 0
;;       global.get $global_env
;;       call $create_closure
;;       global.set $G_double
;;   )
;;   (start $__init)  ;; Runs once at module load
;;
;;   (func $main (export "main")
;;       ;; Now main is clean - just use global.get $G_double
;;       global.get $G_double
;;       local.set $f
;;       ...
;;   )
;;
;; ============================================

;; fn add(a, b) {
;;     return a + b
;; }

;; // Direct call - should use fast path
;; let x = add(3, 4)

;; // Closure call - should still work
;; let f = add
;; let y = f(5, 6)

;; // Result
;; let result = x + y  // 7 + 11 = 18
;; print(result)
(module
    ;; ============================================
    ;; IMPORTS
    ;; ============================================
    (func $log (import "imports" "log") (param i32))

    ;; ============================================
    ;; TYPES
    ;; ============================================
    ;; All functions will follow this signature:
    ;; - param 1: env_ptr (i32) - pointer to captured environment
    ;; - param 2: arg_struct_ptr (i32) - pointer to arguments array
    ;; - result: i32 - pointer to heap-allocated result
    (type $function_type (func (param i32 i32) (result i32)))

    ;; ============================================
    ;; MEMORY & GLOBALS
    ;; ============================================
    (memory $heap 1)
    (global $heap_ptr (mut i32) (i32.const 1024))
    (global $global_env (mut i32) (i32.const 0))

    ;; ============================================
    ;; FUNCTION TABLE
    ;; ============================================
    (table $closures 5 funcref)
    (elem (i32.const 0) $double)

    ;; ============================================
    ;; RUNTIME HELPERS - Environment
    ;; ============================================
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

    ;; ============================================
    ;; APPLICATION FUNCTIONS
    ;; ============================================
    (func $double (param $env_ptr i32) (param $arg_struct_ptr i32) (result i32)
        (local $x i32)
        ;; gets the ptr to X
        local.get $arg_struct_ptr
        i32.const 0
        call $arg_get
        i32.load
        call $untag_integer
        local.set $x
        local.get $x
        local.get $x
        i32.add

        call $create_integer
    )

    ;; ============================================
    ;; RUNTIME HELPERS - Tagged Values
    ;; ============================================
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

    ;; ============================================
    ;; RUNTIME HELPERS - Memory Allocation
    ;; ============================================
    ;; alloc memory
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

    ;; ============================================
    ;; RUNTIME HELPERS - Arguments
    ;; ============================================
    (func $create_arg (param $length i32) (result i32)
        i32.const 4
        i32.const 4
        local.get $length
        i32.mul
        i32.add

        call $alloc
    )

    (func $arg_set (param $arg_struct_ptr i32) (param $idx i32) (param $value i32) (result i32)
        ;; This leave the memory address on the stack
        i32.const 4
        i32.const 4
        local.get $idx
        i32.mul
        i32.add
        local.get $arg_struct_ptr
        i32.add

        local.get $value
        i32.store

        local.get $arg_struct_ptr 
    )

    (func $arg_length (param $arg_struct_ptr i32) (result i32)
        local.get $arg_struct_ptr
        i32.load
    )

    (func $arg_get (param $arg_struct_ptr i32) (param $idx i32) (result i32)
    ;; lets just do a dynamic check here in the future, for now we leave it
        local.get $arg_struct_ptr
        i32.const 4
        i32.const 4
        local.get $idx
        i32.mul
        i32.add
        i32.add

        i32.load
    )

    ;; get value from env given an index
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

    ;; Set value in the env for a given index
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

    ;; ============================================
    ;; RUNTIME HELPERS - Closures
    ;; ============================================
    (func $call_closure (param $closure_ptr i32) (param $arg_struct_ptr i32) (result i32)
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
        local.get $arg_struct_ptr
        local.get $func_idx
        call_indirect  (type $function_type)
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

    ;; ============================================
    ;; MAIN ENTRY POINT
    ;; ============================================
    ;; NOTE: Currently doing initialization here (not ideal!)
    ;; Better: Use (start $__init) pattern shown at top of file
    (func $main (export "main")
        (local $f i32)
        (local $result i32)
        (local $double i32)

        ;; create global env
        ;; declared functions can be stored here
        i32.const 0
        i32.const 1
        call $create_env

        global.set $global_env

        ;; set double
        i32.const 0
        global.get $global_env
        call $create_closure 
        local.set $double

        local.get $double
        local.set $f

        local.get $f

        i32.const 1
        call $create_arg
        i32.const 0
        i32.const 5
        call $create_integer
        call $arg_set

        call $call_closure

        local.set $result

        local.get $result
        call $get_data
        call $untag_integer
        call $log
    )
)

;; fn add(a, b) {
;;     return a + b
;; }

;; let x = add(3, 4)      // Direct call
;; let f = add            // Get closure
;; let y = f(5, 6)        // Closure call
;; let result = x + y     // Should be 18
;; print(result)
