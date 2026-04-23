(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32) (result i32)))
  (type (;2;) (func (param i32 i32 i32)))
  (type (;3;) (func (result i32)))
  (type (;4;) (func (param i32 i32 i32) (result i32)))
  (type (;5;) (func))
  (table (;0;) 5 funcref)
  (memory (;0;) 1)
  (global (;0;) (mut i32) i32.const 1024)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 3)
  (global (;3;) (mut i32) i32.const 2)
  (global (;4;) (mut i32) i32.const 1)
  (global (;5;) (mut i32) i32.const 0)
  (export "memory" (memory 0))
  (export "main" (func 41))
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
    global.get 5
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
  (func (;10;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32)
    local.get 0
    i32.load offset=4
    i32.const 3
    i32.shr_u
    local.set 2
    block ;; label = @1
      loop ;; label = @2
        local.get 3
        local.get 2
        i32.ge_u
        br_if 1 (;@1;)
        local.get 0
        i32.const 8
        local.get 3
        i32.const 8
        i32.mul
        i32.add
        i32.add
        i32.load
        local.set 4
        local.get 4
        local.get 1
        call 24
        call 7
        if ;; label = @3
          local.get 3
          return
        end
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        br 0 (;@2;)
      end
    end
    i32.const -1
  )
  (func (;11;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32)
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 2
    local.get 1
    i32.const 8
    i32.mul
    i32.add
    local.set 3
    global.get 2
    local.get 3
    call 1
    local.set 4
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
    local.get 4
  )
  (func (;12;) (type 3) (result i32)
    global.get 2
    i32.const 0
    call 1
  )
  (func (;13;) (type 0) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    call 10
    local.tee 2
    i32.const 0
    i32.lt_s
    if ;; label = @1
      i32.const 0
      call 6
      return
    end
    local.get 0
    i32.const 8
    local.get 2
    i32.const 8
    i32.mul
    i32.add
    i32.add
    i32.load offset=4
  )
  (func (;14;) (type 4) (param i32 i32 i32) (result i32)
    (local i32 i32)
    local.get 0
    local.get 1
    call 10
    local.tee 3
    i32.const 0
    i32.ge_s
    if (result i32) ;; label = @1
      local.get 0
      i32.const 8
      local.get 3
      i32.const 8
      i32.mul
      i32.add
      i32.add
      local.get 2
      i32.store offset=4
      local.get 0
    else
      local.get 0
      i32.const 1
      call 11
      local.set 0
      local.get 0
      i32.load offset=4
      i32.const 8
      i32.sub
      local.set 4
      local.get 0
      i32.const 8
      local.get 4
      i32.add
      i32.add
      local.get 1
      i32.store
      local.get 0
      i32.const 8
      local.get 4
      i32.add
      i32.add
      local.get 2
      i32.store offset=4
      local.get 0
    end
  )
  (func (;15;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    call 10
    i32.const 0
    i32.ge_s
    call 6
  )
  (func (;16;) (type 1) (param i32) (result i32)
    i32.const 4
    i32.const 4
    local.get 0
    i32.mul
    i32.add
    call 0
  )
  (func (;17;) (type 4) (param i32 i32 i32) (result i32)
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
  (func (;18;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 4
    i32.const 4
    local.get 1
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;19;) (type 1) (param i32) (result i32)
    local.get 0
    i32.load
  )
  (func (;20;) (type 0) (param i32 i32) (result i32)
    (local i32)
    i32.const 12
    call 0
    local.set 2
    local.get 2
    global.get 3
    i32.store
    local.get 2
    local.get 0
    i32.store offset=4
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
  )
  (func (;21;) (type 0) (param i32 i32) (result i32)
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
  (func (;22;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.lt_s
    call 6
  )
  (func (;23;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.gt_s
    call 6
  )
  (func (;24;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32)
    local.get 0
    call 8
    local.get 1
    call 8
    i32.and
    if ;; label = @1
      local.get 0
      local.get 1
      i32.eq
      call 6
      return
    end
    local.get 0
    call 9
    local.set 2
    local.get 1
    call 9
    local.set 3
    local.get 2
    local.get 3
    i32.and
    i32.eqz
    if ;; label = @1
      i32.const 0
      call 6
      return
    end
    local.get 0
    local.get 1
    i32.eq
    if ;; label = @1
      i32.const 1
      call 6
      return
    end
    local.get 0
    i32.load
    local.tee 4
    global.get 5
    i32.ne
    if ;; label = @1
      i32.const 0
      call 6
      return
    end
    local.get 1
    i32.load
    local.get 4
    i32.ne
    if ;; label = @1
      i32.const 0
      call 6
      return
    end
    local.get 0
    local.get 1
    call 35
  )
  (func (;25;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.ne
    call 6
  )
  (func (;26;) (type 0) (param i32 i32) (result i32)
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
  (func (;27;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    i32.const 4
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;28;) (type 4) (param i32 i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    i32.const 4
    i32.mul
    i32.add
    i32.add
    local.get 2
    i32.store
    local.get 2
  )
  (func (;29;) (type 0) (param i32 i32) (result i32)
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
  (func (;30;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.sub
    call 6
  )
  (func (;31;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.mul
    call 6
  )
  (func (;32;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 7
    local.get 1
    call 7
    i32.div_s
    call 6
  )
  (func (;33;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    i32.const 8
    local.get 1
    i32.add
    call 0
    local.set 2
    local.get 2
    global.get 5
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
  (func (;34;) (type 0) (param i32 i32) (result i32)
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
    global.get 5
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
  (func (;35;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32)
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 1
    i32.load offset=4
    local.set 3
    local.get 2
    local.get 3
    i32.ne
    if ;; label = @1
      i32.const 0
      call 6
      return
    end
    block ;; label = @1
      loop ;; label = @2
        local.get 4
        local.get 2
        i32.ge_u
        br_if 1 (;@1;)
        local.get 0
        i32.const 8
        local.get 4
        i32.add
        i32.add
        i32.load8_u
        local.get 1
        i32.const 8
        local.get 4
        i32.add
        i32.add
        i32.load8_u
        i32.ne
        if ;; label = @3
          i32.const 0
          call 6
          return
        end
        local.get 4
        i32.const 1
        i32.add
        local.set 4
        br 0 (;@2;)
      end
    end
    i32.const 1
    call 6
  )
  (func (;36;) (type 1) (param i32) (result i32)
    global.get 4
    local.get 0
    i32.const 4
    i32.mul
    call 1
  )
  (func (;37;) (type 0) (param i32 i32) (result i32)
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
  (func (;38;) (type 4) (param i32 i32 i32) (result i32)
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
  (func (;39;) (type 0) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    i32.load
    local.set 2
    local.get 2
    global.get 4
    i32.eq
    if (result i32) ;; label = @1
      local.get 0
      local.get 1
      call 37
    else
      local.get 2
      i32.const 3
      i32.eq
      if (result i32) ;; label = @2
        local.get 0
        local.get 1
        call 13
      else
        i32.const 0
      end
    end
  )
  (func (;40;) (type 4) (param i32 i32 i32) (result i32)
    (local i32)
    local.get 0
    i32.load
    local.set 3
    local.get 3
    global.get 4
    i32.eq
    if (result i32) ;; label = @1
      local.get 0
      local.get 1
      local.get 2
      call 38
    else
      local.get 3
      i32.const 3
      i32.eq
      if (result i32) ;; label = @2
        local.get 0
        local.get 1
        local.get 2
        call 14
      else
        local.get 0
      end
    end
  )
  (func (;41;) (type 5)
    (local i32 i32)
    global.get 1
    i32.const 0
    call 26
    global.set 1
    call 12
    i32.const 0
    i32.const 4
    call 33
    i32.const 1
    call 6
    call 14
    i32.const 4
    i32.const 9
    call 33
    i32.const 2
    call 6
    call 14
    local.set 0
    local.get 0
    i32.const 0
    i32.const 4
    call 33
    call 39
    local.set 1
  )
  (data (;0;) (i32.const 0) "test")
  (data (;1;) (i32.const 4) "other_key")
)
