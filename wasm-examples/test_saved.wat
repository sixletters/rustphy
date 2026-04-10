(module
   ;; Tag/Untag Helper Functions
   (func $tag_int (param $val i32) (result i32)
      local.get $val
      i32.const 2
      i32.shl
   )
   (func $untag_int (param $tagged i32) (result i32)
      local.get $tagged
      i32.const 2
      i32.shr_s
   )
   (func $tag_bool (param $val i32) (result i32)
      local.get $val
      i32.const 2
      i32.shl
      i32.const 1
      i32.or
   )
   (func $untag_bool (param $tagged i32) (result i32)
      local.get $tagged
      i32.const 2
      i32.shr_s
   )
   (func $tag_pointer (param $addr i32) (result i32)
      local.get $addr
      i32.const 2
      i32.shl
      i32.const 2
      i32.or
   )
   (func $untag_pointer (param $tagged i32) (result i32)
      local.get $tagged
      i32.const 2
      i32.shr_u
   )
   (func $get_tag (param $val i32) (result i32)
      local.get $val
      i32.const 3
      i32.and
   )
   (func $is_int (param $val i32) (result i32)
      local.get $val
      call $get_tag
      i32.const 0
      i32.eq
   )
   (func $is_bool (param $val i32) (result i32)
      local.get $val
      call $get_tag
      i32.const 1
      i32.eq
   )
   (func $is_pointer (param $val i32) (result i32)
      local.get $val
      call $get_tag
      i32.const 2
      i32.eq
   )

   ;; Arithmetic Operations on Tagged Values
   (func $add_values (param $a i32) (param $b i32) (result i32)
      local.get $a
      call $untag_int
      local.get $b
      call $untag_int
      i32.add
      call $tag_int
   )
   (func $sub_values (param $a i32) (param $b i32) (result i32)
      local.get $a
      call $untag_int
      local.get $b
      call $untag_int
      i32.sub
      call $tag_int
   )
   (func $mul_values (param $a i32) (param $b i32) (result i32)
      local.get $a
      call $untag_int
      local.get $b
      call $untag_int
      i32.mul
      call $tag_int
   )
   (func $div_values (param $a i32) (param $b i32) (result i32)
      local.get $a
      call $untag_int
      local.get $b
      call $untag_int
      i32.div_s
      call $tag_int
   )

   (func $main (result i32)
      (local $y i32)
      (local $w i32)
      (local $i i32)

      i32.const 1
      call $tag_bool
      local.set $y

      i32.const 50
      call $tag_int
      local.set $w

      local.get $y
      if
         i32.const 0
         call $tag_bool
         local.set $y
      end
      local.get $w
      local.get $y
      call $add

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
            i32.const 10
            i32.ge
         )
      )
   )
   (func $add (param $y i32) (param $z i32) (result i32)
      (local $other i32)
      local.get $y
      local.get $z
      call $add_values
      local.set $other

      local.get $y
      local.get $z
      call $add_values
      i32.const 5
      call $tag_int
      call $add_values
      local.get $other
      call $add_values
      return
   )


   (func (export "_start")
      call $main
      drop
   )
)
