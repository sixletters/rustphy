(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32) (result i32)))
  (type (;2;) (func (param i32 i32 i32)))
  (type (;3;) (func (param i32 i32 i32) (result i32)))
  (type (;4;) (func))
  (table (;0;) 5 funcref)
  (memory (;0;) 1)
  (global (;0;) (mut i32) i32.const 1024)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 2)
  (global (;3;) (mut i32) i32.const 1)
  (global (;4;) (mut i32) i32.const 0)
  (export "memory" (memory 0))
  (export "main" (func 32))
  (elem (;0;) (i32.const 0) func 34)
  (func (;0;) (type 1) (param i32) (result i32)
    (local i32)
    global.get 0
    local.set 1
    global.get 0
    local.get 0
    i32.add
    global.set 0
    local.get 1
  )
  (func (;1;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    i32.const 8
    local.get 1
    i32.add
    local.set 3
    local.get 3
    call 0
    local.set 2
    local.get 2
    local.get 0
    i32.store
    local.get 2
    local.get 1
    i32.store offset=4
    local.get 2
  )
  (func (;2;) (type 1) (param i32) (result i32)
    (local i32)
    global.get 4
    local.get 0
    call 1
    local.set 1
    local.get 1
  )
  (func (;3;) (type 2) (param i32 i32 i32)
    local.get 0
    i32.const 8
    i32.add
    local.get 1
    i32.add
    local.get 2
    i32.store8
  )
  (func (;4;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 8
    i32.add
    local.get 1
    i32.add
    i32.load8_u
  )
  (func (;5;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load offset=4
  )
  (func (;6;) (type 1) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.shl
    i32.const 1
    i32.or
  )
  (func (;7;) (type 1) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.shr_s
  )
  (func (;8;) (type 1) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.and
  )
  (func (;9;) (type 1) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.and
    i32.eqz
  )
  (func (;10;) (type 1) (param i32) (result i32)
    i32.const 4
    i32.const 4
    local.get 0
    i32.mul
    i32.add
    call 0
  )
  (func (;11;) (type 3) (param i32 i32 i32) (result i32)
    i32.const 4
    i32.const 4
    local.get 1
    i32.mul
    i32.add
    local.get 0
    i32.add
    local.get 2
    i32.store
    local.get 0
  )
  (func (;12;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 4
    i32.const 4
    local.get 1
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;13;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load
  )
  (func (;14;) (type 0) (param i32 i32) (result i32)
    (local i32)
    i32.const 12
    call 0
    local.set 2
    local.get 2
    global.get 2
    i32.store
    local.get 2
    local.get 0
    i32.store offset=4
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
  )
  (func (;15;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 0
    i32.load offset=8
    local.set 3
    local.get 3
    local.get 1
    local.get 2
    call_indirect (type 0)
  )
  (func (;16;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.lt_s
    call 6
  )
  (func (;17;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.gt_s
    call 6
  )
  (func (;18;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.eq
    call 6
  )
  (func (;19;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.ne
    call 6
  )
  (func (;20;) (type 0) (param i32 i32) (result i32)
    (local i32)
    i32.const 8
    local.get 1
    i32.const 4
    i32.mul
    i32.add
    call 0
    local.set 2
    local.get 2
    local.get 0
    i32.store
    local.get 2
    local.get 1
    i32.store offset=4
    local.get 2
  )
  (func (;21;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    i32.const 4
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;22;) (type 2) (param i32 i32 i32)
    local.get 0
    i32.const 8
    local.get 1
    i32.const 4
    i32.mul
    i32.add
    i32.add
    local.get 2
    i32.store
  )
  (func (;23;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 8
    local.get 1
    call 8
    i32.and
    if (result i32) ;; label = @1
      local.get 0
      call 7
      local.get 1
      call 7
      i32.add
      call 6
    else
      i32.const 0
    end
  )
  (func (;24;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.sub
    call 6
  )
  (func (;25;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.mul
    call 6
  )
  (func (;26;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.div_s
    call 6
  )
  (func (;27;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    i32.const 8
    local.get 1
    i32.add
    call 0
    local.set 2
    local.get 2
    global.get 4
    i32.store
    local.get 2
    local.get 1
    i32.store offset=4
    block ;; label = @1
      loop ;; label = @2
        local.get 3
        local.get 1
        i32.ge_u
        br_if 1 (;@1;)
        local.get 2
        i32.const 8
        local.get 3
        i32.add
        i32.add
        local.get 0
        local.get 3
        i32.add
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        br 0 (;@2;)
      end
    end
    local.get 2
  )
  (func (;28;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32)
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 1
    i32.load offset=4
    local.set 3
    i32.const 8
    local.get 2
    local.get 3
    i32.add
    i32.add
    call 0
    local.set 4
    local.get 4
    global.get 4
    i32.store
    local.get 4
    local.get 2
    local.get 3
    i32.add
    i32.store offset=4
    block ;; label = @1
      loop ;; label = @2
        local.get 5
        local.get 2
        i32.ge_u
        br_if 1 (;@1;)
        local.get 4
        i32.const 8
        local.get 5
        i32.add
        i32.add
        local.get 0
        i32.const 8
        local.get 5
        i32.add
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const 1
        i32.add
        local.set 5
        br 0 (;@2;)
      end
    end
    i32.const 0
    local.set 5
    block ;; label = @1
      loop ;; label = @2
        local.get 5
        local.get 3
        i32.ge_u
        br_if 1 (;@1;)
        local.get 4
        i32.const 8
        local.get 2
        local.get 5
        i32.add
        i32.add
        i32.add
        local.get 1
        i32.const 8
        local.get 5
        i32.add
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const 1
        i32.add
        local.set 5
        br 0 (;@2;)
      end
    end
    local.get 4
  )
  (func (;29;) (type 1) (param i32) (result i32)
    global.get 3
    local.get 0
    i32.const 4
    i32.mul
    call 1
  )
  (func (;30;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    call 7
    i32.const 4
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;31;) (type 3) (param i32 i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    call 7
    i32.const 4
    i32.mul
    i32.add
    i32.add
    local.get 2
    i32.store
    local.get 0
  )
  (func (;32;) (type 4)
    (local i32 i32)
    global.get 1
    i32.const 1
    call 20
    global.set 1
    global.get 1
    i32.const 0
    i32.const 1
    call 6
    call 22
    i32.const 0
    global.get 1
    call 14
    local.set 0
    local.get 0
    i32.const 2
    call 10
    i32.const 0
    i32.const 5
    call 6
    call 11
    i32.const 1
    i32.const 10
    call 6
    call 11
    call 15
    local.set 1
  )
  (func (;33;) (type 3) (param i32 i32 i32) (result i32)
    local.get 0
    i32.const 0
    call 20
    local.get 0
    i32.load
    i32.const 0
    call 21
    local.get 1
    call 23
    local.get 2
    call 23
    return
  )
  (func (;34;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.const 0
    call 12
    local.get 1
    i32.const 1
    call 12
    call 33
  )
)
