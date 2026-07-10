(module
  ;; sentiment-analyzer: counts positive/negative words
  ;; export: analyze(text_ptr, text_len) -> sentiment_score
  ;; score > 0 = positive, < 0 = negative, 0 = neutral
  
  (memory (export "memory") 1)
  
  (func (export "analyze") (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $score i32)
    (local $char i32)
    
    (local.set $i (i32.const 0))
    (local.set $score (i32.const 0))
    
    (block $done
      (loop $scan
        (br_if $done (i32.ge_s (local.get $i) (local.get $len)))
        
        (local.set $char 
          (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
        
        ;; '+' (43) increments score, '-' (45) decrements
        (if (i32.eq (local.get $char) (i32.const 43))
          (then 
            (local.set $score (i32.add (local.get $score) (i32.const 1)))))
        (if (i32.eq (local.get $char) (i32.const 45))
          (then 
            (local.set $score (i32.sub (local.get $score) (i32.const 1)))))
        
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $scan)
      )
    )
    
    (local.get $score)
  )
  
  (func (export "version") (result i32)
    (i32.const 1)
  )
)
