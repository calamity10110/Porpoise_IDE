(module
  ;; lint-checker: scans text for common code smells
  ;; export: check(text_ptr, text_len) -> issue_count
  
  (memory (export "memory") 1)
  
  (func (export "check") (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $issues i32)
    (local $prev i32)
    
    (local.set $i (i32.const 0))
    (local.set $issues (i32.const 0))
    (local.set $prev (i32.const 0))
    
    (block $done
      (loop $scan
        (br_if $done (i32.ge_s (local.get $i) (local.get $len)))
        
        ;; detect trailing whitespace (space at EOL)
        (if 
          (i32.and
            (i32.eq (i32.load8_u (i32.add (local.get $ptr) (local.get $i))) (i32.const 32))
            (i32.eq (local.get $prev) (i32.const 32)))
          (then
            (local.set $issues (i32.add (local.get $issues) (i32.const 1)))))
        
        (local.set $prev 
          (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $scan)
      )
    )
    
    (local.get $issues)
  )
  
  (func (export "version") (result i32)
    (i32.const 1)
  )
  
  (func (export "name") (result i32)
    (i32.const 1)
  )
)
