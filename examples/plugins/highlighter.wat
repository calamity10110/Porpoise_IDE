(module
  ;; highlighter: wraps matched patterns with ANSI color codes
  ;; export: highlight(text_ptr, text_len) -> colored_len
  
  (memory (export "memory") 1)
  
  (func (export "highlight") (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $out_len i32)
    
    (local.set $i (i32.const 0))
    (local.set $out_len (local.get $len))
    
    (block $done
      (loop $scan
        (br_if $done (i32.ge_s (local.get $i) (local.get $len)))
        
        ;; check for "error" keyword
        (if (i32.eq (i32.load8_u (local.get $ptr)) (i32.const 101))
          (then
            (local.set $out_len 
              (i32.add (local.get $out_len) (i32.const 9)))
          )
        )
        
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $scan)
      )
    )
    
    (local.get $out_len)
  )
  
  (func (export "version") (result i32)
    (i32.const 1)
  )
)
