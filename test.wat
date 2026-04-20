(module
(memory $heap (export "memory") 1)
(global $heap_ptr (mut i32) (i32.const 1024))
(global $global_env_ptr (mut i32) (i32.const 0))
(type $function_type (func (param i32 i32) (result i32)))

(table $closures 5 funcref)
(global $TYPE_CLOSURE (mut i32) (i32.const 2))
(global $TYPE_ARRAY (mut i32) (i32.const 1))
(global $TYPE_STRING (mut i32) (i32.const 0))
;; Heap Allocation (Bump Allocator)
(func $alloc (param $size i32) (result i32)
   (local $ptr i32)
   ;; Save current heap pointer
   global.get $heap_ptr
   local.set $ptr
   
   ;; Bump heap pointer forward
   global.get $heap_ptr
   local.get $size
   i32.add
   global.set $heap_ptr
   
   ;; Return old pointer
   local.get $ptr
)
(func $heap_alloc (param $type_tag i32) (param $size i32) (result i32)
   (local $ptr i32)
   (local $total_size i32)
   ;; Calculate total size: 8 bytes header + size
   i32.const 8
   local.get $size
   i32.add
   local.set $total_size
   
   ;; Allocate memory
   local.get $total_size
   call $alloc
   local.set $ptr
   
   ;; Write type tag at offset 0
   local.get $ptr
   local.get $type_tag
   i32.store
   
   ;; Write size at offset 4
   local.get $ptr
   local.get $size
   i32.store offset=4
   
   ;; Return pointer to object
   local.get $ptr
)
(func $alloc_string (param $length i32) (result i32)
   (local $ptr i32)
   ;; Allocate string object (TYPE_STRING, length)
   global.get $TYPE_STRING
   local.get $length
   call $heap_alloc
   local.set $ptr
   
   ;; Return pointer
   local.get $ptr
)
(func $string_set (param $ptr i32) (param $index i32) (param $byte i32)
   ;; Calculate address: ptr + 8 (header) + index
   local.get $ptr
   i32.const 8
   i32.add
   local.get $index
   i32.add
   ;; Store byte
   local.get $byte
   i32.store8
)
(func $string_get (param $ptr i32) (param $index i32) (result i32)
   ;; Calculate address: ptr + 8 (header) + index
   local.get $ptr
   i32.const 8
   i32.add
   local.get $index
   i32.add
   ;; Load byte
   i32.load8_u
)
(func $string_length (param $ptr i32) (result i32)
   ;; Load length from offset 4
   local.get $ptr
   i32.load offset=4
)

;; Tag/Untag Helper Functions (1-bit LSB scheme)
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
(func $is_immediate (param $value i32) (result i32)
   local.get $value
   i32.const 1
   i32.and
)
(func $is_pointer (param $value i32) (result i32)
   local.get $value
   i32.const 1
   i32.and
   i32.eqz
)

;; Argument Struct Helpers (closure calling convention)
(func $create_arg (param $length i32) (result i32)
   ;; Allocate 4 (header) + length * 4 bytes
   i32.const 4
   i32.const 4
   local.get $length
   i32.mul
   i32.add
   call $alloc
)
(func $arg_set (param $arg_struct_ptr i32) (param $idx i32) (param $value i32) (result i32)
   ;; Address = arg_struct_ptr + 4 + idx * 4
   i32.const 4
   i32.const 4
   local.get $idx
   i32.mul
   i32.add
   local.get $arg_struct_ptr
   i32.add
   local.get $value
   i32.store
   
   ;; Return arg_struct_ptr for chaining
   local.get $arg_struct_ptr
)
(func $arg_get (param $arg_struct_ptr i32) (param $idx i32) (result i32)
   ;; Address = arg_struct_ptr + 4 + idx * 4
   local.get $arg_struct_ptr
   i32.const 4
   i32.const 4
   local.get $idx
   i32.mul
   i32.add
   i32.add
   i32.load
)
(func $arg_length (param $arg_struct_ptr i32) (result i32)
   local.get $arg_struct_ptr
   i32.load
)

;; Closure Helpers
(func $create_closure (param $func_idx i32) (param $env_ptr i32) (result i32)
   (local $closure_ptr i32)
   ;; Allocate 12 bytes: [type_tag][func_idx][env_ptr]
   i32.const 12
   call $alloc
   local.set $closure_ptr
   
   ;; Store TYPE_CLOSURE at offset 0
   local.get $closure_ptr
   global.get $TYPE_CLOSURE
   i32.store
   
   ;; Store func_idx at offset 4
   local.get $closure_ptr
   local.get $func_idx
   i32.store offset=4
   
   ;; Store env_ptr at offset 8
   local.get $closure_ptr
   local.get $env_ptr
   i32.store offset=8
   
   local.get $closure_ptr
)
(func $call_closure (param $closure_ptr i32) (param $arg_struct_ptr i32) (result i32)
   (local $func_idx i32)
   (local $env i32)
   ;; Load func_idx from offset 4
   local.get $closure_ptr
   i32.load offset=4
   local.set $func_idx
   
   ;; Load env_ptr from offset 8
   local.get $closure_ptr
   i32.load offset=8
   local.set $env
   
   ;; Dispatch: call_indirect expects (env_ptr, arg_struct_ptr, func_idx)
   local.get $env
   local.get $arg_struct_ptr
   local.get $func_idx
   call_indirect (type $function_type)
)

;; Comparison Operations on Tagged Values
(func $lt_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   call $untag_immediate
   local.get $b
   call $untag_immediate
   i32.lt_s
   call $tag_immediate
)
(func $gt_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   call $untag_immediate
   local.get $b
   call $untag_immediate
   i32.gt_s
   call $tag_immediate
)
(func $eq_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   local.get $b
   i32.eq
   call $tag_immediate
)
(func $ne_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   local.get $b
   i32.ne
   call $tag_immediate
)

;; Environment Helpers (lexical scope chains)
(func $create_env (param $parent_ptr i32) (param $var_count i32) (result i32)
   (local $env_ptr i32)
   ;; Allocate 8 (header) + var_count * 4 bytes
   i32.const 8
   local.get $var_count
   i32.const 4
   i32.mul
   i32.add
   call $alloc
   local.set $env_ptr
   
   ;; Store parent_ptr at offset 0
   local.get $env_ptr
   local.get $parent_ptr
   i32.store
   
   ;; Store var_count at offset 4
   local.get $env_ptr
   local.get $var_count
   i32.store offset=4
   
   local.get $env_ptr
)
(func $env_get (param $env_ptr i32) (param $index i32) (result i32)
   ;; Address = env_ptr + 8 + index * 4
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
   ;; Address = env_ptr + 8 + index * 4
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

;; Arithmetic Operations on Tagged Values
(func $add_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   call $is_immediate
   local.get $b
   call $is_immediate
   i32.and
   if (result i32)
      local.get $a
      call $untag_immediate
      local.get $b
      call $untag_immediate
      i32.add
      call $tag_immediate
   else
      ;; heap object addition not yet supported
      i32.const 0
   end
)
(func $sub_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   call $untag_immediate
   local.get $b
   call $untag_immediate
   i32.sub
   call $tag_immediate
)
(func $mul_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   call $untag_immediate
   local.get $b
   call $untag_immediate
   i32.mul
   call $tag_immediate
)
(func $div_values (param $a i32) (param $b i32) (result i32)
   local.get $a
   call $untag_immediate
   local.get $b
   call $untag_immediate
   i32.div_s
   call $tag_immediate
)

;; String Helpers
(func $create_string (param $data_ptr i32) (param $length i32) (result i32)
   (local $str_ptr i32)
   (local $i i32)
   ;; Allocate 8-byte header + length bytes
   i32.const 8
   local.get $length
   i32.add
   call $alloc
   local.set $str_ptr
   
   ;; Store TYPE_STRING at offset 0
   local.get $str_ptr
   global.get $TYPE_STRING
   i32.store
   
   ;; Store byte length at offset 4
   local.get $str_ptr
   local.get $length
   i32.store offset=4
   
   ;; Copy bytes from data_ptr into the string body
   (block $done
      (loop $copy
         local.get $i
         local.get $length
         i32.ge_u
         br_if $done
   
         local.get $str_ptr
         i32.const 8
         local.get $i
         i32.add
         i32.add
         local.get $data_ptr
         local.get $i
         i32.add
         i32.load8_u
         i32.store8
   
         local.get $i
         i32.const 1
         i32.add
         local.set $i
         br $copy
      )
   )
   
   local.get $str_ptr
)
(func $string_concat (param $str1 i32) (param $str2 i32) (result i32)
   (local $len1 i32)
   (local $len2 i32)
   (local $new_str i32)
   (local $i i32)
   ;; Load lengths from each string header (offset 4)
   local.get $str1
   i32.load offset=4
   local.set $len1
   local.get $str2
   i32.load offset=4
   local.set $len2
   
   ;; Allocate 8 (header) + len1 + len2 bytes
   i32.const 8
   local.get $len1
   local.get $len2
   i32.add
   i32.add
   call $alloc
   local.set $new_str
   
   ;; Store TYPE_STRING at offset 0
   local.get $new_str
   global.get $TYPE_STRING
   i32.store
   
   ;; Store combined length at offset 4
   local.get $new_str
   local.get $len1
   local.get $len2
   i32.add
   i32.store offset=4
   
   ;; Copy str1 bytes
   (block $done1
      (loop $loop1
         local.get $i
         local.get $len1
         i32.ge_u
         br_if $done1
         local.get $new_str
         i32.const 8
         local.get $i
         i32.add
         i32.add
         local.get $str1
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
         br $loop1
      )
   )
   
   ;; Reset i and copy str2 bytes
   i32.const 0
   local.set $i
   (block $done2
      (loop $loop2
         local.get $i
         local.get $len2
         i32.ge_u
         br_if $done2
         local.get $new_str
         i32.const 8
         local.get $len1
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

;; Array Helpers
(func $create_array_empty (param $count i32) (result i32)
   ;; Allocate array: TYPE_ARRAY, size=count*4
   global.get $TYPE_ARRAY
   local.get $count
   i32.const 4
   i32.mul
   call $heap_alloc
)
(func $array_get (param $arr_ptr i32) (param $idx_tagged i32) (result i32)
   ;; Calculate address: arr_ptr + 8 + (idx * 4)
   local.get $arr_ptr
   i32.const 8
   local.get $idx_tagged
   call $untag_immediate
   i32.const 4
   i32.mul
   i32.add
   i32.add
   
   ;; Load tagged value
   i32.load
)
(func $array_set (param $arr_ptr i32) (param $idx_tagged i32) (param $val i32) (result i32)
   ;; Calculate address: arr_ptr + 8 + (idx * 4)
   local.get $arr_ptr
   i32.const 8
   local.get $idx_tagged
   call $untag_immediate
   i32.const 4
   i32.mul
   i32.add
   i32.add
   
   ;; Store tagged value
   local.get $val
   i32.store
   
   ;; Return arr_ptr for chaining
   local.get $arr_ptr
)

(func $main (export "main")
   (local $other i32)
   (local $z i32)
   global.get $global_env_ptr
   i32.const 1
   call $create_env
   global.set $global_env_ptr
   global.get $global_env_ptr
   i32.const 0
   i32.const 1
   call $tag_immediate
   call $env_set
   i32.const 0
   global.get $global_env_ptr
   call $create_closure
   local.set $other
   local.get $other
   i32.const 2
   call $create_arg
   i32.const 0
   i32.const 5
   call $tag_immediate
   call $arg_set
   i32.const 1
   i32.const 10
   call $tag_immediate
   call $arg_set
   call $call_closure
   local.set $z
)

(elem (table $closures) (i32.const 0) func $test_closure )
(func $test_direct (param $env_ptr i32) (param $y i32) (param $z i32) (result i32)
   local.get $env_ptr
   i32.const 0
   call $create_env
   local.get $env_ptr
i32.load

i32.const 0

call $env_get

local.get $y
call $add_values

local.get $z
call $add_values

return


)

(func $test_closure (param $env_ptr i32) (param $arg_struct_ptr i32) (result i32)
   local.get $env_ptr
   local.get $arg_struct_ptr
   i32.const 0
   call $arg_get
   local.get $arg_struct_ptr
   i32.const 1
   call $arg_get
   call $test_direct
)

)
