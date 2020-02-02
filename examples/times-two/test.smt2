(declare-const x (_ BitVec 32))

(assert (not (= (bvshl x (_ bv1 32)) (bvadd x x))))

(check-sat)  ; check satisfiability
