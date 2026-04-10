;; ;; (module 
;; ;;     (import "console" "log" (func $log (param i32 i32 i32)))
;; ;;     (import "js" "mem" (memory 1))
;; ;;     (import "js" "mem1" (memory 1))
;; ;;     (memory $mem2 1)
;; ;;     (export "memory2" (memory $mem2))
;; ;;     (global $g (import "js" "global") (mut i32))
;; ;;     (func $getAnswer (result i32) 
;; ;;         i32.const 42
;; ;;     )
;; ;;     (func (export "getAnswerPlus1") (result i32)
;; ;;         call $getAnswer
;; ;;         i32.const 2
;; ;;         i32.add
;; ;;     )

;; ;;     (func (export "getGlobal") (result i32)
;; ;;         (global.get $g)
;; ;;     )
;; ;;     (data (i32.const 0) "Hi")
;; ;;     (func (export "incGlobal")
;; ;;         global.get $g
;; ;;         i32.const 1
;; ;;         i32.add
;; ;;         global.set $g
;; ;;     )
;; ;;     (func (export "writeHi")
;; ;;         i32.const 0
;; ;;         i32.const 2
;; ;;         call $log
;; ;;     )

;; ;;     (data (memory 1) (i32.const 0) "Memory 1 data")
;; ;;     (data (memory 2) (i32.const 0) "Memory 2 data")
;; ;; )
;; ;; Simple WASI Print Example
;; (module
;;     ;; Import WASI fd_write function
;;     (import "wasi_snapshot_preview1" "fd_write"
;;         (func $fd_write (param i32 i32 i32 i32) (result i32)))

;;     ;; Allocate memory
;;     (memory 1)
;;     (export "memory" (memory 0))

;;     ;; Store "Hello from WASI!\n" starting at offset 8
;;     (data (i32.const 8) "Hello from WASI!\n")

;;     ;; Entry point for standalone execution
;;     (func (export "_start")
;;         ;; Build iovec structure at memory offset 0
;;         ;; iovec.ptr = 8 (where our string is)
;;         i32.const 0          ;; Address to store iovec.ptr
;;         i32.const 8          ;; Value: pointer to string
;;         i32.store            ;; memory[0..4] = 8

;;         ;; iovec.len = 17 (length of "Hello from WASI!\n")
;;         i32.const 4          ;; Address to store iovec.len
;;         i32.const 17         ;; Value: string length
;;         i32.store            ;; memory[4..8] = 17

;;         ;; Call fd_write(fd, iovs, iovs_len, nwritten)
;;         i32.const 1          ;; fd = 1 (stdout)
;;         i32.const 0          ;; iovs = pointer to iovec array
;;         i32.const 1          ;; iovs_len = 1 (one iovec)
;;         i32.const 20         ;; nwritten = where to store bytes written
;;         call $fd_write       ;; Make the call
;;         drop                 ;; Drop return value (errno)
;;     )
;; )
(module
   (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
   (memory (export "memory") 1)
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

   (func $print (param $val i32) (result i32)
      (local $num i32)
      (local $iov_ptr i32)
      (local $written i32)
      local.get $val
      call $untag_int
      local.set $num
      i32.const 0
      i32.const 16
      i32.store
      i32.const 4
      i32.const 1
      i32.store
      i32.const 16
      i32.const 10
      i32.store8
      i32.const 1
      i32.const 0
      i32.const 1
      i32.const 8
      call $fd_write
   )
   (func $main (result i32)
      (local $y i32)
      (local $w i32)

      i32.const 10
      call $tag_int
      local.set $y

      i32.const 5
      call $tag_int
      local.set $w

      local.get $w
      local.get $y
      call $x
   )
   (func $x (param $y i32) (param $z i32) (result i32)
      local.get $y
      local.get $z
      call $add_values
   )


   (func (export "_start")
      call $main
      call $print
      drop
   )
)
