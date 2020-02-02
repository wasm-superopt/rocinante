(declare-const x (_ BitVec 32))

; assert( (x << 1) != (x + x)
(assert (not (= (bvshl x (_ bv1 32)) (bvadd x x))))

; returns unsat
(check-sat)  ; check satisfiability
