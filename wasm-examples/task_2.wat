(module
    ;; ============================================
    ;; IMPORTS
    ;; ============================================
    (func $log (import "imports" "log") (param i32))


    ;; ============================================
    ;; TYPES
    ;; ============================================
    ;; All functions will follow this signature if they are a closure:
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
    (data (i32.const 0) "harris")

    ;; Types on the heap
    ;; stored in object header
    ;; memory layout
    ;; offset 0: Type tag (0 = string)
    ;; offset 4: Length (number of bytes)
    ;; offset 8: Character data (UTF-8 bytes)
    ;; Example: "hello"
    ;; [0, 5, "h", "e", "l", "l", "l", "0]"
    (global $TYPE_STRING (mut i32) (i32.const 0))

    ;; Example: [1, 2, 3]
    ;; [1, 5, 3, tagged(1), tagged(2), tagged(3), 0, 0]
    ;; for arrays u have the capacity (length) and then followed by the
    ;; actual length of the array
    (global $TYPE_ARRAY (mut i32) (i32.const 1))
    ;; Example: closure of idx 5 and env ptr 6
    ;; [2, 5, 6]
    (global $TYPE_CLOSURE (mut i32) (i32.const 2))


    ;; ============================================
    ;; FUNCTION TABLE
    ;; ============================================
    (table $closures 5 funcref)
    (elem (i32.const 0) $add_closure)

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
        i32.load offset=4
        local.set $func_idx

        ;; get the env pointr
        local.get $closure_ptr
        i32.load offset=8
        local.set $env

        ;; call via table 
        local.get $env
        local.get $arg_struct_ptr
        local.get $func_idx
        call_indirect  (type $function_type)
    )

    ;; A closure is of the form [2 (4 bytes), function_idx (4 bytes), env pointer (4 bytes)]
    (func $create_closure (param $func_idx i32) (param $env_ptr i32) (result i32)
        (local $closure_ptr i32)

        ;; allocate 8 bytes on the heap
        i32.const 12
        call $alloc
        
        ;; set the closure_ptr
        local.set $closure_ptr

        ;; store closure tag
        local.get $closure_ptr
        i32.const 2
        i32.store 

        ;; store func idx
        local.get $closure_ptr
        local.get $func_idx
        i32.store offset=4

        ;; store env ptr
        local.get $closure_ptr
        local.get $env_ptr
        i32.store offset=8

        local.get $closure_ptr
    )

    ;; create string
    (func $create_string (param $data_ptr i32) (param $length i32) (result i32)
        (local $str_ptr i32)
        (local $i i32)

        ;; allocate 8 bytes for header + length bytes
        i32.const 8
        local.get $length
        i32.add
        call $alloc
        local.set $str_ptr

        ;; Store type tag
        local.get $str_ptr
        i32.const 0  ;; TYPE_STRING
        i32.store

        ;; store length
        local.get $str_ptr
        local.get $length
        i32.store offset=4

        ;; store the character data
        (block $done
            (loop $copy
                local.get $i
                local.get $length  
                i32.ge_u ;; Check if i >= length
                br_if $done ;; break out of "done" loop
                
                ;; Copy one byte
                local.get $str_ptr
                i32.const 8
                local.get $i ;; add 8 to i to offset header
                i32.add
                i32.add
                
                local.get $data_ptr
                local.get $i
                i32.add ;; add i to data_ptr
                i32.load8_u  ;; Load 1 byte
                i32.store8    ;; Store 1 byte
                
                local.get $i ;; increment i
                i32.const 1
                i32.add
                local.set $i
                br $copy ;; go back to copy
            )
        )

        local.get $str_ptr
    )

    (func $string_length (param $str_ptr i32) (result i32)
        local.get $str_ptr
        i32.load offset=4
    )

    (func $string_concat (param $str1 i32) (param $str2 i32) (result i32)
        ;; take in to strings that are pointers to the actual structure
        (local $length1 i32)
        (local $length2 i32)
        (local $new_str i32)
        (local $i i32)

        local.get $str1
        call $string_length
        local.set $length1

        local.get $str2
        call $string_length
        local.set $length2

        ;; allocate new string
        i32.const 8
        local.get $length1
        local.get $length2
        i32.add
        i32.add
        call $alloc
        local.set $new_str

        ;; set string tag type
        local.get $new_str
        global.get $TYPE_STRING
        i32.store

        local.get $new_str
        local.get $length1
        local.get $length2
        i32.add
        i32.store offset=4

        (block $done1
            (loop $loop1
                local.get $i
                local.get $length1
                i32.ge_u
                br_if $done1

                ;; current ptr to copy to
                local.get $new_str
                i32.const 8
                local.get $i
                i32.add
                i32.add

                ;; current ptr to copy from
                local.get $str1
                i32.const 8
                local.get $i
                i32.add
                i32.add
                i32.load8_u

                ;; this stores the loaded value into the pointer above
                i32.store8

                local.get $i
                i32.const 1
                i32.add
                local.set $i
                br $loop1
            )
        )

        ;; Copy str2
        i32.const 0
        local.set $i
        (block $done2
            (loop $loop2
                local.get $i
                local.get $length2
                i32.ge_u
                br_if $done2
                
                local.get $new_str
                i32.const 8
                local.get $length1
                local.get $i
                i32.add
                i32.add
                i32.add
                
                local.get $str2
                i32.const 8
                local.get $i
                i32.add
                i32.add
                i32.load8_u
                i32.store8
                
                local.get $i
                i32.const 1
                i32.add
                local.set $i
                br $loop2
            )
        )

        local.get $new_str
    )

    ;; Lets do nan-boxing/pointer taggin
    ;; we wanna use 1 bit of i32 to encode type + data
    ;; Better - use full i32:
    ;;   - If value & 1 == 0: it's a pointer (aligned addresses are even)
    ;;   - If value & 1 == 1: it's an immediate integer (val >> 1)
    ;; we will only have 31 bits left for the actual data
    ;; this is level 1 of tagging
    (func $tag_immediate (param $value i32) (result i32)
        local.get $value
        i32.const 1
        i32.shl
        i32.const 1
        i32.or
    )

    (func $untag_immediate (param $value i32) (result i32)
        local.get $value
        i32.const 1
        i32.shr_s
    )

    ;; Check if value is immediate: val & 1 == 1
    (func $is_immediate (param $value i32) (result i32)
        ;; Return 1 if immediate, 0 if pointer
        local.get $value
        i32.const 1
        i32.and
    )

    ;; Check if value is pointer: val & 1 == 0
    (func $is_pointer (param $value i32) (result i32)
        ;; Return 1 if pointer, 0 if immediate
        local.get $value
        i32.const 1
        i32.and
        i32.eqz    ;; Returns 1 if zero (pointer), 0 if one (immediate)
    )

    (func $add_values (param $a i32) (param $b i32) (result i32)
        local.get $a
        call $is_immediate

        local.get $b
        call $is_immediate

        i32.and
        ;; both are immediates
        if (result i32)
            ;; Fast path
            ;; this could be optimized by just removing the tags
            local.get $a
            call $untag_immediate

            local.get $b
            call $untag_immediate

            i32.add
            call $tag_immediate
        else
            ;; Since we do not have addition of heap objects yet
            i32.const 5
        end
    )
    
    ;; In the fugure all functions, even if their signature match should have an env
    ;; since they can in theory reference variables outside
    (func $add_direct (param $env_ptr i32) (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        call $add_values
    )

    (func $add_closure (param $env_ptr i32) (param $arg_struct_ptr i32) (result i32)
        local.get $env_ptr
    
        ;; Unpack arg 0
        local.get $arg_struct_ptr
        i32.const 0
        call $arg_get
        
        ;; Unpack arg 1
        local.get $arg_struct_ptr
        i32.const 1
        call $arg_get

        ;; Call the direct version (fast!)
        call $add_direct
    )

    ;; Now we do the dual representation, which is what V8 does
    ;; we have a function generated that is the actual function signature
    ;; and a closure version
    (func $main (export "main") (result i32) 
        (local $x i32)
        (local $f i32)
        (local $y i32)
        (local $constant i32)
        (local $result i32)
        (local $mystring i32)

        global.get $global_env
        i32.const 3
        call $tag_immediate
        i32.const 4
        call $tag_immediate
        call $add_direct
        local.set $x

        i32.const 0
        global.get $global_env
        call $create_closure
        local.set $f

        local.get $f
        i32.const 2
        call $create_arg

        i32.const 0
        i32.const 5
        call $tag_immediate
        call $arg_set

        i32.const 1
        i32.const 6
        call $tag_immediate   
        call $arg_set
        call $call_closure

        local.set $y

        local.get $x
        local.get $y
        call $add_values

        call $untag_immediate
        call $log

        i32.const 0
        i32.const 6
        call $create_string
        local.set $x

        ;; Now you can use x
        local.get $x
        call $string_length
        call $log         ;; Should print 6

        i32.const 1
    )
)
