(module
  (type $function_type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32) (result i32)))
  (type (;2;) (func (param i32 i32 i32)))
  (type (;3;) (func (param i32 i32 i32) (result i32)))
  (type (;4;) (func (result i32)))
  (type (;5;) (func (param i32 i32) (result i32 i32)))
  (data (i32.const 0) "harris")
  (func $alloc (;0;) (type 1) (param $size i32) (result i32)
    (local $ptr i32)
    global.get $heap_ptr
    local.set $ptr
    global.get $heap_ptr
    local.get $size
    i32.add
    global.set $heap_ptr
    local.get $ptr
  )
  (func $heap_alloc (;1;) (type $function_type) (param $type_tag i32) (param $size i32) (result i32)
    (local $ptr i32) (local $total_size i32)
    i32.const 8
    local.get $size
    i32.add
    local.set $total_size
    local.get $total_size
    call $alloc
    local.set $ptr
    local.get $ptr
    local.get $type_tag
    i32.store
    local.get $ptr
    local.get $size
    i32.store offset=4
    local.get $ptr
  )
  (func $alloc_string (;2;) (type 1) (param $length i32) (result i32)
    (local $ptr i32)
    i32.const 0
    local.get $length
    call $heap_alloc
    local.set $ptr
    local.get $ptr
  )
  (func $string_set (;3;) (type 2) (param $ptr i32) (param $index i32) (param $byte i32)
    local.get $ptr
    i32.const 8
    i32.add
    local.get $index
    i32.add
    local.get $byte
    i32.store8
  )
  (func $string_get (;4;) (type $function_type) (param $ptr i32) (param $index i32) (result i32)
    local.get $ptr
    i32.const 8
    i32.add
    local.get $index
    i32.add
    i32.load8_u
  )
  (func $string_length (;5;) (type 1) (param $ptr i32) (result i32)
    local.get $ptr
    i32.load offset=4
  )
  (func $tag_immediate (;6;) (type 1) (param $value i32) (result i32)
    local.get $value
    i32.const 1
    i32.shl
    i32.const 1
    i32.or
  )
  (func $untag_immediate (;7;) (type 1) (param $value i32) (result i32)
    local.get $value
    i32.const 1
    i32.shr_s
  )
  (func $is_immediate (;8;) (type 1) (param $value i32) (result i32)
    local.get $value
    i32.const 1
    i32.and
  )
  (func $is_pointer (;9;) (type 1) (param $value i32) (result i32)
    local.get $value
    i32.const 1
    i32.and
    i32.eqz
  )
  (func $create_arg (;10;) (type 1) (param $length i32) (result i32)
    i32.const 4
    i32.const 4
    local.get $length
    i32.mul
    i32.add
    call $alloc
  )
  (func $arg_set (;11;) (type 3) (param $arg_struct_ptr i32) (param $idx i32) (param $value i32) (result i32)
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
  (func $arg_get (;12;) (type $function_type) (param $arg_struct_ptr i32) (param $idx i32) (result i32)
    local.get $arg_struct_ptr
    i32.const 4
    i32.const 4
    local.get $idx
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func $arg_length (;13;) (type 1) (param $arg_struct_ptr i32) (result i32)
    local.get $arg_struct_ptr
    i32.load
  )
  (func $create_closure (;14;) (type $function_type) (param $func_idx i32) (param $env_ptr i32) (result i32)
    (local $closure_ptr i32)
    i32.const 12
    call $alloc
    local.set $closure_ptr
    local.get $closure_ptr
    i32.const 2
    i32.store
    local.get $closure_ptr
    local.get $func_idx
    i32.store offset=4
    local.get $closure_ptr
    local.get $env_ptr
    i32.store offset=8
    local.get $closure_ptr
  )
  (func $call_closure (;15;) (type $function_type) (param $closure_ptr i32) (param $arg_struct_ptr i32) (result i32)
    (local $func_idx i32) (local $env i32)
    local.get $closure_ptr
    i32.load offset=4
    local.set $func_idx
    local.get $closure_ptr
    i32.load offset=8
    local.set $env
    local.get $env
    local.get $arg_struct_ptr
    local.get $func_idx
    call_indirect (type $function_type)
  )
  (func $lt_values (;16;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    call $untag_immediate
    local.get $b
    call $untag_immediate
    i32.lt_s
    call $tag_immediate
  )
  (func $gt_values (;17;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    call $untag_immediate
    local.get $b
    call $untag_immediate
    i32.gt_s
    call $tag_immediate
  )
  (func $eq_values (;18;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.eq
    call $tag_immediate
  )
  (func $ne_values (;19;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.ne
    call $tag_immediate
  )
  (func $create_env (;20;) (type $function_type) (param $parent_ptr i32) (param $var_count i32) (result i32)
    (local $env_ptr i32)
    i32.const 8
    local.get $var_count
    i32.const 4
    i32.mul
    i32.add
    call $alloc
    local.set $env_ptr
    local.get $env_ptr
    local.get $parent_ptr
    i32.store
    local.get $env_ptr
    local.get $var_count
    i32.store offset=4
    local.get $env_ptr
  )
  (func $env_get (;21;) (type $function_type) (param $env_ptr i32) (param $index i32) (result i32)
    local.get $env_ptr
    i32.const 8
    local.get $index
    i32.const 4
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func $env_set (;22;) (type 2) (param $env_ptr i32) (param $index i32) (param $value i32)
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
  (func $add_values (;23;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    call $is_immediate
    local.get $b
    call $is_immediate
    i32.and
    if (result i32) ;; label = @1
      local.get $a
      call $untag_immediate
      local.get $b
      call $untag_immediate
      i32.add
      call $tag_immediate
    else
      i32.const 0
    end
  )
  (func $sub_values (;24;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    call $untag_immediate
    local.get $b
    call $untag_immediate
    i32.sub
    call $tag_immediate
  )
  (func $mul_values (;25;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    call $untag_immediate
    local.get $b
    call $untag_immediate
    i32.mul
    call $tag_immediate
  )
  (func $div_values (;26;) (type $function_type) (param $a i32) (param $b i32) (result i32)
    local.get $a
    call $untag_immediate
    local.get $b
    call $untag_immediate
    i32.div_s
    call $tag_immediate
  )
  (func $create_string (;27;) (type $function_type) (param $data_ptr i32) (param $length i32) (result i32)
    (local $str_ptr i32) (local $i i32)
    i32.const 8
    local.get $length
    i32.add
    call $alloc
    local.set $str_ptr
    local.get $str_ptr
    i32.const 0
    i32.store
    local.get $str_ptr
    local.get $length
    i32.store offset=4
    block $done
      loop $copy
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
      end
    end
    local.get $str_ptr
  )
  (func $string_concat (;28;) (type $function_type) (param $str1 i32) (param $str2 i32) (result i32)
    (local $len1 i32) (local $len2 i32) (local $new_str i32) (local $i i32)
    local.get $str1
    i32.load offset=4
    local.set $len1
    local.get $str2
    i32.load offset=4
    local.set $len2
    i32.const 8
    local.get $len1
    local.get $len2
    i32.add
    i32.add
    call $alloc
    local.set $new_str
    local.get $new_str
    i32.const 0
    i32.store
    local.get $new_str
    local.get $len1
    local.get $len2
    i32.add
    i32.store offset=4
    block $done1
      loop $loop1
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
      end
    end
    i32.const 0
    local.set $i
    block $done2
      loop $loop2
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
      end
    end
    local.get $new_str
  )
  (func $main (;29;) (type 4) (result i32)
    (local $y i32) (local $w i32)
    i32.const 50
    call $tag_immediate
    local.set $y
    i32.const 50
    call $tag_immediate
    local.set $w
    local.get $y
    i32.const 2
    call $tag_immediate
    call $add_direct

    ;; lets say we want to do let y = [1,2,"harris"]
    i32.const 3
    call $create_array_empty

    i32.const 0
    call $tag_immediate
    i32.const 1
    call $tag_immediate
    call $array_set

    i32.const 1
    call $tag_immediate
    i32.const 2
    call $tag_immediate
    call $array_set

    i32.const 2
    call $tag_immediate
    i32.const 0
    i32.const 6
    call $create_string
    call $array_set

    local.set $y
  )
  (func $add_direct (;30;) (type $function_type) (param $y i32) (param $z i32) (result i32)
    local.get $y
    i32.const 10
    call $tag_immediate
    i32.add
    local.set $z
    local.get $z
    i32.const 10
    call $tag_immediate
    i32.const 1
    call $tag_immediate
    if ;; label = @1
      i32.const 5
      call $tag_immediate
      return
    else
      local.get $z
      return
    end
    i32.const 5
    call $tag_immediate
    local.set $y

    ;; example of how to do a for loop for the following
    ;; lets say we have something like 
    ;; let i = 0
    ;; for(i < 10) {
    ;;    i = i + 1;
    ;;    log(i);
    ;; }
    ;; this would be equals to the following wasm code
    i32.const 0
    local.set $i
    (block $done_loop
        (loop $do_loop
          local.get $i
          call $untag_immediate
          i32.const 10
          i32.ge
          ;; result of expression on top of stack
          ;; might have to negate the results here
          ;; we are essentially exiting the loop only when
          ;; the condition is not met
          ;; but this leave a true condition on the stack
          br_if $done_loop

          local.get $i
          call $untag_immediate
          i32.const 1

          i32.add 
          call $tag_immeidate
          local.set $i

          local.get $i
          call $log

          br $do_loop
        )
    )
  )

  (func $create_array_empty (param $count i32) (result i32)
    i32.const 1
    local.get $count
    i32.const 4
    i32.mul
    call $heap_alloc
  )

  (func $array_get (param $arr_ptr i32) (param $idx_tagged i32) (result i32)
    ;; todo: throw error when idx > size of arra
    local.get $arr_ptr
    i32.const 8
    local.get $idx_tagged
    call $untag_immediate
    i32.const 4
    i32.mul
    i32.add
    i32.add
    i32.load
  )

  (func $array_set (param $arr_ptr i32) (param $idx_tagged i32) (param $val i32) (result i32)
    local.get $arr_ptr
    i32.const 8
    local.get $idx_tagged
    call $untag_immediate
    i32.const 4
    i32.mul
    i32.add
    i32.add
    local.get $val
    i32.store ;; expects stack to have [addr, value]
    local.get $arr_ptr
  )

  (func $add_closure (;31;) (type 5) (param $env_ptr i32) (param $arg_struct_ptr i32) (result i32 i32)
    local.get $env_ptr
    local.get $arg_struct_ptr
    i32.const 0
    call $arg_get
    local.get $arg_struct_ptr
    i32.const 1
    call $arg_get
    call $add_direct
  )
  (table $closures (;0;) 5 funcref)
  (memory $heap (;0;) 1)
  (global $heap_ptr (;0;) (mut i32) i32.const 1024)
  (global $global_env (;1;) (mut i32) i32.const 0)
  (global $TYPE_CLOSURE (;2;) (mut i32) i32.const 2)
  (global $TYPE_ARRAY (;3;) (mut i32) i32.const 1)
  (global $TYPE_STRING (;4;) (mut i32) i32.const 0)
  (export "main" (func $main))
)
