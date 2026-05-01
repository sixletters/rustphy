(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32)))
  (type (;2;) (func (param i32) (result i32)))
  (type (;3;) (func (param i32 i32 i32)))
  (type (;4;) (func (result i32)))
  (type (;5;) (func (param i32 i32 i32) (result i32)))
  (type (;6;) (func))
  (import "env" "log" (func (;0;) (type 1)))
  (table (;0;) 5 funcref)
  (memory (;0;) 1)
  (global (;0;) (mut i32) i32.const 1024)
  (global (;1;) (mut i32) i32.const 0)
  (global (;2;) (mut i32) i32.const 3)
  (global (;3;) (mut i32) i32.const 2)
  (global (;4;) (mut i32) i32.const 1)
  (global (;5;) (mut i32) i32.const 0)
  (export "memory" (memory 0))
  (export "main" (func 42))
  (func (;1;) (type 2) (param i32) (result i32)
    (local i32)
    global.get 0
    local.set 1
    global.get 0
    local.get 0
    i32.add
    global.set 0
    local.get 1
  )
  (func (;2;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    i32.const 8
    local.get 1
    i32.add
    local.set 3
    local.get 3
    call 1
    local.set 2
    local.get 2
    local.get 0
    i32.store
    local.get 2
    local.get 1
    i32.store offset=4
    local.get 2
  )
  (func (;3;) (type 2) (param i32) (result i32)
    (local i32)
    global.get 5
    local.get 0
    call 2
    local.set 1
    local.get 1
  )
  (func (;4;) (type 3) (param i32 i32 i32)
    local.get 0
    i32.const 8
    i32.add
    local.get 1
    i32.add
    local.get 2
    i32.store8
  )
  (func (;5;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 8
    i32.add
    local.get 1
    i32.add
    i32.load8_u
  )
  (func (;6;) (type 2) (param i32) (result i32)
    local.get 0
    i32.load offset=4
  )
  (func (;7;) (type 2) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.shl
    i32.const 1
    i32.or
  )
  (func (;8;) (type 2) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.shr_s
  )
  (func (;9;) (type 2) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.and
  )
  (func (;10;) (type 2) (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.and
    i32.eqz
  )
  (func (;11;) (type 0) (param i32 i32) (result i32)
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
        call 25
        call 8
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
  (func (;12;) (type 0) (param i32 i32) (result i32)
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
    call 2
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
  (func (;13;) (type 4) (result i32)
    global.get 2
    i32.const 0
    call 2
  )
  (func (;14;) (type 0) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    call 11
    local.tee 2
    i32.const 0
    i32.lt_s
    if ;; label = @1
      i32.const 0
      call 7
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
  (func (;15;) (type 5) (param i32 i32 i32) (result i32)
    (local i32 i32)
    local.get 0
    local.get 1
    call 11
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
      call 12
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
  (func (;16;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    call 11
    i32.const 0
    i32.ge_s
    call 7
  )
  (func (;17;) (type 2) (param i32) (result i32)
    i32.const 4
    i32.const 4
    local.get 0
    i32.mul
    i32.add
    call 1
  )
  (func (;18;) (type 5) (param i32 i32 i32) (result i32)
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
  (func (;19;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 4
    i32.const 4
    local.get 1
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;20;) (type 2) (param i32) (result i32)
    local.get 0
    i32.load
  )
  (func (;21;) (type 0) (param i32 i32) (result i32)
    (local i32)
    i32.const 12
    call 1
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
  (func (;22;) (type 0) (param i32 i32) (result i32)
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
  (func (;23;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 8
    local.get 1
    call 8
    i32.lt_s
    call 7
  )
  (func (;24;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 8
    local.get 1
    call 8
    i32.gt_s
    call 7
  )
  (func (;25;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32)
    local.get 0
    call 9
    local.get 1
    call 9
    i32.and
    if ;; label = @1
      local.get 0
      local.get 1
      i32.eq
      call 7
      return
    end
    local.get 0
    call 10
    local.set 2
    local.get 1
    call 10
    local.set 3
    local.get 2
    local.get 3
    i32.and
    i32.eqz
    if ;; label = @1
      i32.const 0
      call 7
      return
    end
    local.get 0
    local.get 1
    i32.eq
    if ;; label = @1
      i32.const 1
      call 7
      return
    end
    local.get 0
    i32.load
    local.tee 4
    global.get 5
    i32.ne
    if ;; label = @1
      i32.const 0
      call 7
      return
    end
    local.get 1
    i32.load
    local.get 4
    i32.ne
    if ;; label = @1
      i32.const 0
      call 7
      return
    end
    local.get 0
    local.get 1
    call 36
  )
  (func (;26;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.ne
    call 7
  )
  (func (;27;) (type 0) (param i32 i32) (result i32)
    (local i32)
    i32.const 8
    local.get 1
    i32.const 4
    i32.mul
    i32.add
    call 1
    local.set 2
    local.get 2
    local.get 0
    i32.store
    local.get 2
    local.get 1
    i32.store offset=4
    local.get 2
  )
  (func (;28;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    i32.const 4
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;29;) (type 5) (param i32 i32 i32) (result i32)
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
  (func (;30;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 9
    local.get 1
    call 9
    i32.and
    if (result i32) ;; label = @1
      local.get 0
      call 8
      local.get 1
      call 8
      i32.add
      call 7
    else
      i32.const 0
    end
  )
  (func (;31;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 8
    local.get 1
    call 8
    i32.sub
    call 7
  )
  (func (;32;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 8
    local.get 1
    call 8
    i32.mul
    call 7
  )
  (func (;33;) (type 0) (param i32 i32) (result i32)
    local.get 0
    call 8
    local.get 1
    call 8
    i32.div_s
    call 7
  )
  (func (;34;) (type 0) (param i32 i32) (result i32)
    (local i32 i32)
    i32.const 8
    local.get 1
    i32.add
    call 1
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
  (func (;35;) (type 0) (param i32 i32) (result i32)
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
    call 1
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
  (func (;36;) (type 0) (param i32 i32) (result i32)
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
      call 7
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
          call 7
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
    call 7
  )
  (func (;37;) (type 2) (param i32) (result i32)
    global.get 4
    local.get 0
    i32.const 4
    i32.mul
    call 2
  )
  (func (;38;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    call 8
    i32.const 4
    i32.mul
    i32.add
    i32.add
    i32.load
  )
  (func (;39;) (type 5) (param i32 i32 i32) (result i32)
    local.get 0
    i32.const 8
    local.get 1
    call 8
    i32.const 4
    i32.mul
    i32.add
    i32.add
    local.get 2
    i32.store
    local.get 0
  )
  (func (;40;) (type 0) (param i32 i32) (result i32)
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
      call 38
    else
      local.get 2
      i32.const 3
      i32.eq
      if (result i32) ;; label = @2
        local.get 0
        local.get 1
        call 14
      else
        i32.const 0
      end
    end
  )
  (func (;41;) (type 5) (param i32 i32 i32) (result i32)
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
      call 39
    else
      local.get 3
      i32.const 3
      i32.eq
      if (result i32) ;; label = @2
        local.get 0
        local.get 1
        local.get 2
        call 15
      else
        local.get 0
      end
    end
  )
  (func (;42;) (type 6)
    (local i32)
    global.get 1
    i32.const 0
    call 27
    global.set 1
    i32.const 1
    call 7
    call 8
    if (result i32) ;; label = @1
      i32.const 1
      call 7
      call 8
    else
      i32.const 0
    end
    call 7
    local.set 0
    i32.const 0
    i32.const 6
    call 34
    call 0
    i32.const 0
    drop
  )
  (data (;0;) (i32.const 0) "harris")
)
